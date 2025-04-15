use std::{
    fs::{self, File},
    io::{self, Write},
    path::{Path, PathBuf},
};

use axum::body::StreamBody;
use chrono::Utc;
use sqlx::PgPool;
use tokio::{
    fs::File as TokioFile,
    io::{AsyncReadExt, AsyncWriteExt},
};
use tokio_util::io::ReaderStream;

use crate::{
    error::{ApiError, Result},
    models::{Clip, ClipStatus},
};

/// 剪辑服务
#[derive(Debug, Clone)]
pub struct ClipService {
    pool: PgPool,
    base_dir: PathBuf,
}

impl ClipService {
    /// 创建新的剪辑服务实例
    pub fn new(pool: PgPool, base_dir: impl AsRef<Path>) -> Self {
        let base_dir = base_dir.as_ref().to_path_buf();
        
        // 确保基础目录存在
        fs::create_dir_all(&base_dir).expect("无法创建基础目录");
        
        // 确保临时目录和数据目录存在
        let temp_dir = base_dir.join("temp");
        fs::create_dir_all(&temp_dir).expect("无法创建临时目录");
        
        let data_dir = base_dir.join("data");
        fs::create_dir_all(&data_dir).expect("无法创建数据目录");
        
        Self {
            pool,
            base_dir,
        }
    }
    
    /// 获取数据目录
    pub fn get_data_dir(&self) -> PathBuf {
        self.base_dir.join("data")
    }
    
    /// 获取临时目录
    pub fn get_temp_dir(&self) -> PathBuf {
        self.base_dir.join("temp")
    }

    /// 创建新的剪辑任务
    pub async fn create_clip(
        &self,
        material_path: String,
        output_path: String,
        prompt: String,
    ) -> Result<Clip> {
        let clip = Clip::new(material_path, output_path, prompt);
        
        // 创建剪辑任务目录
        let clip_dir = self.get_clip_dir(&clip.id);
        fs::create_dir_all(&clip_dir)?;
        
        // 创建素材目录
        let material_dir = clip_dir.join("materials");
        fs::create_dir_all(&material_dir)?;
        
        // 创建输出目录
        let output_dir = clip_dir.join("output");
        fs::create_dir_all(&output_dir)?;
        
        // 保存剪辑任务信息到数据库
        sqlx::query(
            r#"
            INSERT INTO clips 
            (id, material_path, output_path, prompt, status, created_at, updated_at, result_files)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
        )
        .bind(&clip.id)
        .bind(&clip.material_path)
        .bind(&clip.output_path)
        .bind(&clip.prompt)
        .bind(clip.status.to_string())
        .bind(clip.created_at)
        .bind(clip.updated_at)
        .bind(serde_json::to_value(&clip.result_files)?)
        .execute(&self.pool)
        .await
        .map_err(|e| ApiError::DatabaseError(e.to_string()))?;
        
        Ok(clip)
    }

    /// 获取剪辑任务
    pub async fn get_clip(&self, clip_id: &str) -> Result<Clip> {
        let row = sqlx::query("SELECT * FROM clips WHERE id = $1")
            .bind(clip_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| ApiError::DatabaseError(e.to_string()))?;
            
        match row {
            Some(row) => {
                Clip::from_row(row)
                    .map_err(|e| ApiError::DatabaseError(e.to_string()))
            },
            None => Err(ApiError::ClipNotFound(clip_id.to_string())),
        }
    }

    /// 获取剪辑任务状态
    pub async fn get_clip_status(&self, clip_id: &str) -> Result<ClipStatus> {
        let status: String = sqlx::query_scalar("SELECT status FROM clips WHERE id = $1")
            .bind(clip_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| ApiError::DatabaseError(e.to_string()))?
            .ok_or_else(|| ApiError::ClipNotFound(clip_id.to_string()))?;
            
        Ok(ClipStatus::from(status.as_str()))
    }

    /// 更新剪辑任务状态
    pub async fn update_clip_status(&self, clip_id: &str, status: ClipStatus) -> Result<()> {
        let now = Utc::now();
        
        let rows_affected = sqlx::query(
            "UPDATE clips SET status = $1, updated_at = $2 WHERE id = $3"
        )
        .bind(status.to_string())
        .bind(now)
        .bind(clip_id)
        .execute(&self.pool)
        .await
        .map_err(|e| ApiError::DatabaseError(e.to_string()))?
        .rows_affected();
        
        if rows_affected == 0 {
            return Err(ApiError::ClipNotFound(clip_id.to_string()));
        }
        
        Ok(())
    }

    /// 直接上传文件（不需要剪辑ID）
    pub async fn upload_file_direct(&self, file_name: &str, data: Vec<u8>) -> Result<String> {
        println!("直接上传文件: file_name={}, 文件大小={}字节", file_name, data.len());
        
        // 获取数据目录
        let data_dir = self.get_data_dir();
        println!("数据目录路径: {}", data_dir.display());
        
        // 确保数据目录存在
        if !data_dir.exists() {
            println!("数据目录不存在，正在创建: {}", data_dir.display());
            fs::create_dir_all(&data_dir)
                .map_err(|e| {
                    println!("创建数据目录失败: {}", e);
                    ApiError::FileOperationFailed(format!("创建数据目录失败: {}", e))
                })?;
            println!("数据目录创建成功");
        }
        
        let file_path = data_dir.join(file_name);
        println!("目标文件路径: {}", file_path.display());
        
        // 写入文件
        println!("开始创建文件");
        let mut file = File::create(&file_path)
            .map_err(|e| {
                println!("创建文件失败 {}: {}", file_path.display(), e);
                ApiError::FileOperationFailed(format!("创建文件失败 {}: {}", file_path.display(), e))
            })?;
        
        println!("开始写入文件数据");    
        file.write_all(&data)
            .map_err(|e| {
                println!("写入文件失败 {}: {}", file_path.display(), e);
                ApiError::FileOperationFailed(format!("写入文件失败 {}: {}", file_path.display(), e))
            })?;
        
        println!("文件上传成功: file_name={}", file_name);
        
        // 生成文件访问链接
        let file_url = format!("/api/download/{}", file_name);
        
        Ok(file_url)
    }
    
    /// 上传文件分块
    pub async fn upload_file_chunk(
        &self, 
        upload_id: &str, 
        chunk_index: usize, 
        total_chunks: usize, 
        data: &[u8]
    ) -> Result<()> {
        println!("上传文件分块: upload_id={}, chunk_index={}/{}, 分块大小={}字节", 
            upload_id, chunk_index, total_chunks, data.len());
        
        // 获取临时目录
        let temp_dir = self.get_temp_dir();
        
        // 创建上传ID目录
        let upload_dir = temp_dir.join(upload_id);
        if !upload_dir.exists() {
            fs::create_dir_all(&upload_dir)
                .map_err(|e| {
                    println!("创建上传目录失败: {}", e);
                    ApiError::FileOperationFailed(format!("创建上传目录失败: {}", e))
                })?;
        }
        
        // 创建分块文件
        let chunk_file_path = upload_dir.join(format!("{}", chunk_index));
        
        // 写入分块数据
        let mut file = File::create(&chunk_file_path)
            .map_err(|e| {
                println!("创建分块文件失败: {}", e);
                ApiError::FileOperationFailed(format!("创建分块文件失败: {}", e))
            })?;
        
        file.write_all(data)
            .map_err(|e| {
                println!("写入分块数据失败: {}", e);
                ApiError::FileOperationFailed(format!("写入分块数据失败: {}", e))
            })?;
        
        println!("分块上传成功: upload_id={}, chunk_index={}", upload_id, chunk_index);
        
        Ok(())
    }
    
    /// 完成分块上传
    pub async fn complete_chunked_upload(
        &self, 
        upload_id: &str, 
        file_name: &str
    ) -> Result<String> {
        println!("完成分块上传: upload_id={}, file_name={}", upload_id, file_name);
        
        // 获取临时目录和数据目录
        let temp_dir = self.get_temp_dir();
        let data_dir = self.get_data_dir();
        
        // 检查上传ID目录是否存在
        let upload_dir = temp_dir.join(upload_id);
        if !upload_dir.exists() {
            return Err(ApiError::InvalidRequest(format!("上传ID不存在: {}", upload_id)));
        }
        
        // 获取所有分块文件
        let mut chunk_files = Vec::new();
        for entry in fs::read_dir(&upload_dir)
            .map_err(|e| ApiError::FileOperationFailed(format!("读取上传目录失败: {}", e)))? {
            let entry = entry
                .map_err(|e| ApiError::FileOperationFailed(format!("读取目录条目失败: {}", e)))?;
            
            let path = entry.path();
            if path.is_file() {
                // 分块文件名是数字索引
                if let Some(file_name) = path.file_name() {
                    if let Some(file_name_str) = file_name.to_str() {
                        if let Ok(index) = file_name_str.parse::<usize>() {
                            chunk_files.push((index, path));
                        }
                    }
                }
            }
        }
        
        // 按索引排序分块
        chunk_files.sort_by_key(|(index, _)| *index);
        
        // 创建目标文件
        let target_file_path = data_dir.join(file_name);
        let mut target_file = tokio::fs::File::create(&target_file_path).await
            .map_err(|e| {
                println!("创建目标文件失败: {}", e);
                ApiError::FileOperationFailed(format!("创建目标文件失败: {}", e))
            })?;
        
        // 合并分块
        let mut total_bytes = 0;
        for (index, chunk_path) in chunk_files {
            println!("处理分块 {}: {}", index, chunk_path.display());
            
            // 读取分块文件
            let mut chunk_file = tokio::fs::File::open(&chunk_path).await
                .map_err(|e| {
                    println!("打开分块文件失败: {}", e);
                    ApiError::FileOperationFailed(format!("打开分块文件失败: {}", e))
                })?;
            
            // 读取并写入数据
            let mut buffer = vec![0u8; 8 * 1024 * 1024]; // 8MB 缓冲区
            loop {
                let bytes_read = chunk_file.read(&mut buffer).await
                    .map_err(|e| {
                        println!("读取分块数据失败: {}", e);
                        ApiError::FileOperationFailed(format!("读取分块数据失败: {}", e))
                    })?;
                
                if bytes_read == 0 {
                    break;
                }
                
                target_file.write_all(&buffer[0..bytes_read]).await
                    .map_err(|e| {
                        println!("写入目标文件失败: {}", e);
                        ApiError::FileOperationFailed(format!("写入目标文件失败: {}", e))
                    })?;
                
                total_bytes += bytes_read;
            }
        }
        
        // 确保所有数据都写入磁盘
        target_file.flush().await
            .map_err(|e| {
                println!("刷新文件数据失败: {}", e);
                ApiError::FileOperationFailed(format!("刷新文件数据失败: {}", e))
            })?;
        
        println!("文件合并完成，总大小: {} 字节", total_bytes);
        
        // 清理临时文件
        tokio::spawn(async move {
            if let Err(e) = tokio::fs::remove_dir_all(upload_dir).await {
                println!("清理临时文件失败: {}", e);
            }
        });
        
        // 生成文件访问链接
        let file_url = format!("/api/download/{}", file_name);
        
        Ok(file_url)
    }
    
    /// 添加结果文件
    pub async fn add_result_file(&self, clip_id: &str, file_name: &str) -> Result<()> {
        // 获取当前结果文件列表
        let clip = self.get_clip(clip_id).await?;
        let mut result_files = clip.result_files.clone();
        
        // 添加新文件
        result_files.push(file_name.to_string());
        
        // 更新数据库
        let now = Utc::now();
        sqlx::query(
            "UPDATE clips SET result_files = $1, updated_at = $2 WHERE id = $3"
        )
        .bind(serde_json::to_value(&result_files)?)
        .bind(now)
        .bind(clip_id)
        .execute(&self.pool)
        .await
        .map_err(|e| ApiError::DatabaseError(e.to_string()))?;
        
        Ok(())
    }

    /// 获取文件流（通用方法，不区分素材文件和结果文件）
    pub async fn get_file_stream(
        &self,
        clip_id: &str,
        file_name: &str,
    ) -> Result<StreamBody<ReaderStream<TokioFile>>> {
        println!("获取文件流: clip_id={}, file_name={}", clip_id, file_name);
        
        // 检查剪辑任务是否存在
        let _clip = self.get_clip(clip_id).await?;
        
        // 获取数据目录
        let data_dir = self.base_dir.join("data");
        
        // 尝试在不同目录中查找文件
        let possible_paths = vec![
            data_dir.join(file_name),                  // 在数据目录下
            self.base_dir.join(file_name),             // 在基础目录下
            self.get_clip_dir(clip_id).join(file_name), // 在剪辑目录下（兼容旧数据）
            self.get_clip_dir(clip_id).join("materials").join(file_name), // 在素材目录下（兼容旧数据）
            self.get_clip_dir(clip_id).join("output").join(file_name),    // 在输出目录下（兼容旧数据）
        ];
        
        // 查找文件
        let file_path = possible_paths.iter()
            .find(|path| path.exists())
            .ok_or_else(|| ApiError::FileNotFound(format!(
                "文件 {} 不存在于任何目录中",
                file_name
            )))?;
        
        println!("找到文件路径: {}", file_path.display());
        
        // 打开文件
        let file = TokioFile::open(file_path).await.map_err(|e| {
            ApiError::FileOperationFailed(format!("无法打开文件: {}", e))
        })?;
        
        // 创建文件流
        let stream = ReaderStream::new(file);
        let body = StreamBody::new(stream);
        
        println!("文件流创建成功");
        Ok(body)
    }

    /// 直接获取文件流（不需要剪辑ID）
    pub async fn get_file_stream_direct(
        &self,
        file_name: &str,
    ) -> Result<StreamBody<ReaderStream<TokioFile>>> {
        println!("直接获取文件流: file_name={}", file_name);
        
        // 获取数据目录
        let data_dir = self.base_dir.join("data");
        
        // 文件路径
        let file_path = data_dir.join(file_name);
        
        // 检查文件是否存在
        if !file_path.exists() {
            return Err(ApiError::FileNotFound(format!(
                "文件 {} 不存在",
                file_name
            )));
        }
        
        println!("找到文件路径: {}", file_path.display());
        
        // 打开文件
        let file = TokioFile::open(file_path).await.map_err(|e| {
            ApiError::FileOperationFailed(format!("无法打开文件: {}", e))
        })?;
        
        // 创建文件流
        let stream = ReaderStream::new(file);
        let body = StreamBody::new(stream);
        
        println!("文件流创建成功");
        Ok(body)
    }
    /// 获取剪辑任务目录
    fn get_clip_dir(&self, clip_id: &str) -> PathBuf {
        self.base_dir.join(clip_id)
    }

    /// 获取所有剪辑任务
    pub async fn get_all_clips(&self) -> Result<Vec<Clip>> {
        let rows = sqlx::query("SELECT * FROM clips ORDER BY created_at DESC")
            .fetch_all(&self.pool)
            .await
            .map_err(|e| ApiError::DatabaseError(e.to_string()))?;
            
        let mut clips = Vec::new();
        for row in rows {
            let clip = Clip::from_row(row)
                .map_err(|e| ApiError::DatabaseError(e.to_string()))?;
            clips.push(clip);
        }
        
        Ok(clips)
    }

    /// 设置素材包链接
    pub async fn set_material_file(&self, clip_id: &str, material_file: &str) -> Result<()> {
        println!("设置素材包链接: clip_id={}, material_file={}", clip_id, material_file);
        
        // 更新数据库
        let now = Utc::now();
        sqlx::query(
            "UPDATE clips SET material_file = $1, updated_at = $2 WHERE id = $3"
        )
        .bind(serde_json::to_value(material_file)?)
        .bind(now)
        .bind(clip_id)
        .execute(&self.pool)
        .await
        .map_err(|e| {
            println!("更新数据库失败: {}", e);
            ApiError::DatabaseError(e.to_string())
        })?;
        
        println!("素材包链接已保存到数据库");
        Ok(())
    }
}
