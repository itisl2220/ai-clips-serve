use std::{
    collections::HashMap,
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
    sync::Arc,
};

use axum::body::StreamBody;
use chrono::Utc;
use sqlx::{PgPool, postgres::PgRow};
use tokio::fs::File as TokioFile;
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
        
        Self {
            pool,
            base_dir,
        }
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

    /// 上传素材文件
    pub async fn upload_file(&self, clip_id: &str, file_name: &str, data: Vec<u8>) -> Result<String> {
        println!("开始上传文件: clip_id={}, file_name={}, 文件大小={}字节", clip_id, file_name, data.len());
        
        // 检查剪辑任务是否存在
        self.get_clip(clip_id).await?;
        println!("剪辑任务存在，继续上传");
        
        // 获取素材目录
        let material_dir = self.get_clip_dir(clip_id).join("materials");
        println!("素材目录路径: {}", material_dir.display());
        
        // 确保素材目录存在
        if !material_dir.exists() {
            println!("素材目录不存在，正在创建: {}", material_dir.display());
            fs::create_dir_all(&material_dir)
                .map_err(|e| {
                    println!("创建素材目录失败: {}", e);
                    ApiError::FileOperationFailed(format!("创建素材目录失败: {}", e))
                })?;
            println!("素材目录创建成功");
        }
        
        let file_path = material_dir.join(file_name);
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
        
        println!("文件上传成功: clip_id={}, file_name={}", clip_id, file_name);
        
        // 生成文件访问链接
        let file_url = format!("/api/download/{}/file?name={}", clip_id, file_name);
        
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
        
        // 获取剪辑任务目录
        let clip_dir = self.get_clip_dir(clip_id);
        
        // 尝试在不同目录中查找文件
        let possible_paths = vec![
            clip_dir.join(file_name),                  // 直接在剪辑目录下
            clip_dir.join("materials").join(file_name), // 在素材目录下
            clip_dir.join("output").join(file_name),    // 在输出目录下
        ];
        
        // 查找文件
        let file_path = possible_paths.iter()
            .find(|path| path.exists())
            .ok_or_else(|| ApiError::FileNotFound(format!(
                "文件 {} 不存在于剪辑任务 {} 的任何目录中",
                file_name, clip_id
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

    /// 处理剪辑任务（模拟AI处理过程）
    pub async fn process_clip(&self, clip_id: &str) -> Result<()> {
        // 获取剪辑任务并检查是否存在
        let _clip = self.get_clip(clip_id).await?;
        
        // 更新状态为处理中
        self.update_clip_status(clip_id, ClipStatus::Processing).await?;
        
        // 模拟AI处理过程（实际项目中这里会调用AI处理逻辑）
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
        
        // 创建一个示例结果文件
        let output_dir = self.get_clip_dir(clip_id).join("output");
        let result_file_name = format!("result_{}.mp4", clip_id);
        let result_file_path = output_dir.join(&result_file_name);
        
        // 写入示例内容（实际项目中这里会保存AI处理结果）
        let mut file = File::create(result_file_path)?;
        file.write_all(b"This is a sample result file")?;
        
        // 添加结果文件到剪辑任务
        self.add_result_file(clip_id, &result_file_name).await?;
        
        // 更新状态为已完成
        self.update_clip_status(clip_id, ClipStatus::Completed).await?;
        
        Ok(())
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
