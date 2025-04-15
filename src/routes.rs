use std::sync::Arc;

use axum::{
    extract::{DefaultBodyLimit, Multipart, Path, Query},
    http::{header, StatusCode},
    response::IntoResponse,
    routing::{get, post, put},
    Extension, Json, Router,
};
use serde::{Deserialize, Serialize};
use tokio::io::AsyncWriteExt;
use uuid::Uuid;

use crate::{
    error::{ApiError, Result},
    models::{ApiResponse, ClipStatus, CreateClipRequest, UpdateMaterialFileRequest},
    services::ClipService,
};

/// 创建API路由
pub fn create_router(clip_service: Arc<ClipService>) -> Router {
    Router::new()
        .route("/clips", post(create_clip))
        .route("/clips", get(get_all_clips))
        .route("/upload", post(upload_file).layer(DefaultBodyLimit::max(5 * 1024 * 1024 * 1024))) // 5 GB limit
        .route("/upload/chunk", post(upload_file_chunk))
        .route("/upload/complete", post(complete_chunked_upload))
        .route("/clips/:clip_id/status", get(get_clip_status))
        .route("/clips/:clip_id/status", put(update_clip_status))
        .route("/clips/:clip_id", get(get_clip))
        .route("/clips/:clip_id/material", post(update_material_file))
        .route("/clips/:clip_id/result", post(add_result_file))
        .route("/download/:clip_id/file", get(download_file))
        .route("/download/file", get(download_file_direct))
        .route("/download/:file_name", get(download_file_by_name))
        .route("/health", get(health_check))
        .layer(Extension(clip_service))
}

/// 健康检查
async fn health_check() -> impl IntoResponse {
    Json(ApiResponse::success("服务器正常运行中"))
}

/// 创建剪辑任务
async fn create_clip(
    Extension(clip_service): Extension<Arc<ClipService>>,
    Json(request): Json<CreateClipRequest>,
) -> Result<impl IntoResponse> {
    let clip = clip_service.create_clip(
        request.material_path,
        request.output_path,
        request.prompt,
    ).await?;    
    Ok(Json(ApiResponse::success(clip)))
}

/// 获取所有剪辑任务
async fn get_all_clips(
    Extension(clip_service): Extension<Arc<ClipService>>,
) -> Result<impl IntoResponse> {
    let clips = clip_service.get_all_clips().await?;
    Ok(Json(ApiResponse::success(clips)))
}

/// 上传素材文件（流式处理）
async fn upload_file(
    Extension(clip_service): Extension<Arc<ClipService>>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse> {
    println!("开始处理文件上传请求（流式）");
    let mut original_file_name = String::new();
    
    // 生成临时文件名
    let temp_uuid = Uuid::new_v4().to_string();
    let mut uuid_file_name = temp_uuid.clone();
    
    // 获取数据目录
    let data_dir = clip_service.get_data_dir();
    
    // 使用更健壮的方式处理multipart表单
    let mut field_processed = false;
    
    while let Some(mut field) = multipart.next_field().await.map_err(|e| {
        println!("解析表单数据失败: {}", e);
        ApiError::InvalidRequest(format!("解析表单数据失败: {}", e))
    })? {
        let name = field.name().unwrap_or("").to_string();
        println!("处理表单字段: {}", name);
        
        if name == "file" {
            println!("读取file字段");
            field_processed = true;
            
            // 获取文件名
            if let Some(filename) = field.file_name() {
                println!("原始文件名: {}", filename);
                original_file_name = filename.to_string();
                
                // 生成UUID作为文件名，但保留原始扩展名
                let extension = original_file_name.split('.').last().unwrap_or("");
                uuid_file_name = if !extension.is_empty() {
                    format!("{}.{}", temp_uuid, extension)
                } else {
                    temp_uuid.clone()
                };
            } else {
                original_file_name = "unknown".to_string();
                println!("未提供文件名，使用默认名称: {}", original_file_name);
            }
            
            // 创建临时文件
            let file_path = data_dir.join(&uuid_file_name);
            println!("创建临时文件: {}", file_path.display());
            
            let mut file = tokio::fs::File::create(&file_path).await.map_err(|e| {
                println!("创建临时文件失败: {}", e);
                ApiError::FileOperationFailed(format!("创建临时文件失败: {}", e))
            })?;
            
            // 流式读取并写入文件
            let mut total_bytes = 0;
            
            while let Some(chunk) = field.chunk().await.map_err(|e| {
                println!("读取文件块失败: {}", e);
                ApiError::FileOperationFailed(format!("读取文件块失败: {}", e))
            })? {
                file.write_all(&chunk).await.map_err(|e| {
                    println!("写入文件块失败: {}", e);
                    ApiError::FileOperationFailed(format!("写入文件块失败: {}", e))
                })?;
                
                total_bytes += chunk.len();
                if total_bytes % (5 * 1024 * 1024) == 0 {  // 每5MB记录一次
                    println!("已处理: {} MB", total_bytes / (1024 * 1024));
                }
            }
            
            // 确保所有数据都写入磁盘
            file.flush().await.map_err(|e| {
                println!("刷新文件数据失败: {}", e);
                ApiError::FileOperationFailed(format!("刷新文件数据失败: {}", e))
            })?;
            
            println!("文件写入完成，总大小: {} 字节", total_bytes);
        }
    }
    
    // 检查是否处理了文件字段
    if !field_processed {
        println!("未找到file字段");
        return Err(ApiError::InvalidRequest("未找到file字段".to_string()));
    }
    
    // 验证必要参数
    if original_file_name.is_empty() {
        println!("缺少文件名");
        return Err(ApiError::InvalidRequest("缺少文件名".to_string()));
    }
    
    println!("生成UUID文件名: {}", uuid_file_name);
    
    // 生成文件访问链接
    let file_url = format!("/api/download/{}", uuid_file_name);
    println!("文件上传成功，链接: {}", file_url);
    
    // 返回成功响应，包含文件链接
    println!("返回成功响应");
    Ok(Json(ApiResponse::success_with_data(
        "文件上传成功",
        serde_json::json!({
            "file_url": file_url,
            "file_name": uuid_file_name,
            "original_file_name": original_file_name,
        })
    )))
}

/// 获取剪辑任务状态
async fn get_clip_status(
    Extension(clip_service): Extension<Arc<ClipService>>,
    Path(clip_id): Path<String>,
) -> Result<impl IntoResponse> {
    let status = clip_service.get_clip_status(&clip_id).await?;
    Ok(Json(ApiResponse::success(status)))
}

/// 更新任务状态请求
#[derive(Debug, Deserialize)]
pub struct UpdateStatusRequest {
    status: String,
}

/// 更新剪辑任务状态
async fn update_clip_status(
    Extension(clip_service): Extension<Arc<ClipService>>,
    Path(clip_id): Path<String>,
    Json(request): Json<UpdateStatusRequest>,
) -> Result<impl IntoResponse> {
    let status = match request.status.as_str() {
        "pending" => ClipStatus::Pending,
        "processing" => ClipStatus::Processing,
        "completed" => ClipStatus::Completed,
        "failed" => ClipStatus::Failed,
        _ => return Err(ApiError::InvalidRequest(format!("无效的状态值: {}", request.status))),
    };
    
    // 如果要将状态设置为已完成，需要验证任务是否有结果文件
    if status == ClipStatus::Completed {
        let clip = clip_service.get_clip(&clip_id).await?;
        if clip.result_files.is_empty() {
            return Err(ApiError::InvalidRequest("没有结果文件的任务不能标记为已完成".to_string()));
        }
    }
    
    clip_service.update_clip_status(&clip_id, status).await?;
    Ok(Json(ApiResponse::success("状态更新成功")))
}

/// 获取剪辑任务详情
async fn get_clip(
    Extension(clip_service): Extension<Arc<ClipService>>,
    Path(clip_id): Path<String>,
) -> Result<impl IntoResponse> {
    let clip = clip_service.get_clip(&clip_id).await?;
    Ok(Json(ApiResponse::success(clip)))
}

/// 下载文件查询参数
#[derive(Debug, Deserialize)]
pub struct DownloadQuery {
    name: String,
}

/// 下载文件
pub async fn download_file(
    Extension(clip_service): Extension<Arc<ClipService>>,
    Path(clip_id): Path<String>,
    Query(query): Query<DownloadQuery>,
) -> Result<impl IntoResponse> {
    let file_name = &query.name;
    println!("请求下载文件: clip_id={}, file_name={}", clip_id, file_name);
    
    // 直接获取文件流，不区分素材文件和结果文件
    let body = clip_service.get_file_stream(&clip_id, file_name).await?;
    
    // 构建响应头
    let mut headers = axum::http::HeaderMap::new();
    headers.insert(header::CONTENT_TYPE, "application/octet-stream".parse().unwrap());
    headers.insert(
        header::CONTENT_DISPOSITION,
        format!("attachment; filename=\"{}\"", file_name).parse().unwrap(),
    );
    
    Ok((StatusCode::OK, headers, body))
}

/// 下载文件（直接）
pub async fn download_file_direct(
    Extension(clip_service): Extension<Arc<ClipService>>,
    Query(query): Query<DownloadQuery>,
) -> Result<impl IntoResponse> {
    let file_name = &query.name;
    println!("请求下载文件: file_name={}", file_name);
    
    // 直接获取文件流，不区分素材文件和结果文件
    let body = clip_service.get_file_stream_direct(file_name).await?;
    
    // 构建响应头
    let mut headers = axum::http::HeaderMap::new();
    headers.insert(header::CONTENT_TYPE, "application/octet-stream".parse().unwrap());
    headers.insert(
        header::CONTENT_DISPOSITION,
        format!("attachment; filename=\"{}\"", file_name).parse().unwrap(),
    );
    
    Ok((StatusCode::OK, headers, body))
}

/// 下载文件（根据文件名）
pub async fn download_file_by_name(
    Extension(clip_service): Extension<Arc<ClipService>>,
    Path(file_name): Path<String>,
) -> Result<impl IntoResponse> {
    println!("请求下载文件: file_name={}", file_name);
    
    // 直接获取文件流，不区分素材文件和结果文件
    let body = clip_service.get_file_stream_direct(&file_name).await?;
    
    // 构建响应头
    let mut headers = axum::http::HeaderMap::new();
    headers.insert(header::CONTENT_TYPE, "application/octet-stream".parse().unwrap());
    headers.insert(
        header::CONTENT_DISPOSITION,
        format!("attachment; filename=\"{}\"", file_name).parse().unwrap(),
    );
    
    Ok((StatusCode::OK, headers, body))
}

/// 修改剪辑任务的素材链接
async fn update_material_file(
    Extension(clip_service): Extension<Arc<ClipService>>,
    Path(clip_id): Path<String>,
    Json(request): Json<UpdateMaterialFileRequest>,
) -> Result<impl IntoResponse> {
    println!("修改素材链接: clip_id={}, material_file={}", clip_id, request.material_file);
    
    // 验证剪辑任务是否存在
    let _clip = clip_service.get_clip(&clip_id).await?;
    
    // 设置素材包链接
    clip_service.set_material_file(&clip_id, &request.material_file).await?;
    
    // 返回成功响应
    Ok(Json(ApiResponse::success_with_data(
        "素材链接修改成功",
        serde_json::json!({
            "clip_id": clip_id,
            "material_file": request.material_file
        })
    )))
}

/// 分块上传请求结构
#[derive(Debug, Deserialize)]
pub struct ChunkUploadRequest {
    /// 分块索引
    chunk_index: usize,
    /// 总分块数
    total_chunks: usize,
    /// 上传ID（首次上传时为空，服务端生成）
    upload_id: Option<String>,
    /// 原始文件名
    original_file_name: String,
}

/// 分块上传响应
#[derive(Debug, Serialize)]
pub struct ChunkUploadResponse {
    /// 上传ID
    upload_id: String,
    /// 已上传分块索引
    chunk_index: usize,
    /// 总分块数
    total_chunks: usize,
    /// 上传进度（百分比）
    progress: f32,
}

/// 完成分块上传请求
#[derive(Debug, Deserialize)]
pub struct CompleteChunkUploadRequest {
    /// 上传ID
    upload_id: String,
    /// 原始文件名
    original_file_name: String,
}

/// 上传文件分块
async fn upload_file_chunk(
    Extension(clip_service): Extension<Arc<ClipService>>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse> {
    println!("开始处理分块上传请求");
    
    // 解析multipart表单数据
    let mut chunk_index = 0;
    let mut total_chunks = 0;
    let mut upload_id = String::new();
    let mut original_file_name = String::new();
    let mut chunk_data = Vec::new();
    
    while let Some(field) = multipart.next_field().await.map_err(|e| {
        println!("解析表单数据失败: {}", e);
        ApiError::InvalidRequest(format!("解析表单数据失败: {}", e))
    })? {
        let name = field.name().unwrap_or("").to_string();
        
        match name.as_str() {
            "chunk_index" => {
                let value = field.text().await.map_err(|e| {
                    ApiError::InvalidRequest(format!("读取chunk_index失败: {}", e))
                })?;
                chunk_index = value.parse().map_err(|e| {
                    ApiError::InvalidRequest(format!("无效的chunk_index: {}", e))
                })?;
            },
            "total_chunks" => {
                let value = field.text().await.map_err(|e| {
                    ApiError::InvalidRequest(format!("读取total_chunks失败: {}", e))
                })?;
                total_chunks = value.parse().map_err(|e| {
                    ApiError::InvalidRequest(format!("无效的total_chunks: {}", e))
                })?;
            },
            "upload_id" => {
                upload_id = field.text().await.map_err(|e| {
                    ApiError::InvalidRequest(format!("读取upload_id失败: {}", e))
                })?;
            },
            "original_file_name" => {
                original_file_name = field.text().await.map_err(|e| {
                    ApiError::InvalidRequest(format!("读取original_file_name失败: {}", e))
                })?;
            },
            "file" => {
                // 读取文件块内容
                chunk_data = field.bytes().await.map_err(|e| {
                    ApiError::FileOperationFailed(format!("读取文件块数据失败: {}", e))
                })?.to_vec();
            },
            _ => {
                println!("忽略未知字段: {}", name);
            }
        }
    }
    
    // 验证必要参数
    if original_file_name.is_empty() || chunk_data.is_empty() {
        return Err(ApiError::InvalidRequest("缺少文件名或文件数据".to_string()));
    }
    
    if total_chunks == 0 {
        return Err(ApiError::InvalidRequest("总分块数不能为0".to_string()));
    }
    
    // 如果是首次上传，生成上传ID
    if upload_id.is_empty() {
        upload_id = Uuid::new_v4().to_string();
        println!("生成新的上传ID: {}", upload_id);
    }
    
    // 上传分块
    clip_service.upload_file_chunk(&upload_id, chunk_index, total_chunks, &chunk_data).await?;
    
    // 计算上传进度
    let progress = (chunk_index as f32 + 1.0) / (total_chunks as f32) * 100.0;
    
    // 返回响应
    let response = ChunkUploadResponse {
        upload_id: upload_id.clone(),
        chunk_index,
        total_chunks,
        progress,
    };
    
    Ok(Json(ApiResponse::success(response)))
}

/// 完成分块上传
async fn complete_chunked_upload(
    Extension(clip_service): Extension<Arc<ClipService>>,
    Json(request): Json<CompleteChunkUploadRequest>,
) -> Result<impl IntoResponse> {
    println!("开始处理完成分块上传请求");
    
    // 验证参数
    if request.upload_id.is_empty() || request.original_file_name.is_empty() {
        return Err(ApiError::InvalidRequest("缺少上传ID或文件名".to_string()));
    }
    
    // 生成UUID作为文件名，但保留原始扩展名
    let extension = request.original_file_name.split('.').last().unwrap_or("");
    let uuid_file_name = if !extension.is_empty() {
        format!("{}.{}", Uuid::new_v4(), extension)
    } else {
        Uuid::new_v4().to_string()
    };
    
    // 合并分块并完成上传
    let file_url = clip_service.complete_chunked_upload(
        &request.upload_id, 
        &uuid_file_name
    ).await?;
    
    // 返回成功响应
    Ok(Json(ApiResponse::success_with_data(
        "文件上传成功",
        serde_json::json!({
            "file_url": file_url,
            "file_name": uuid_file_name,
            "original_file_name": request.original_file_name,
        })
    )))
}

/// 添加结果文件请求
#[derive(Debug, Deserialize)]
pub struct AddResultFileRequest {
    pub file_name: String,
}

/// 添加结果文件到任务
async fn add_result_file(
    Extension(clip_service): Extension<Arc<ClipService>>,
    Path(clip_id): Path<String>,
    Json(request): Json<AddResultFileRequest>,
) -> Result<impl IntoResponse> {
    println!("添加结果文件: clip_id={}, file_name={}", clip_id, request.file_name);
    
    // 验证剪辑任务是否存在
    let _clip = clip_service.get_clip(&clip_id).await?;
    
    // 添加结果文件
    clip_service.add_result_file(&clip_id, &request.file_name).await?;
    
    // 返回成功响应
    Ok(Json(ApiResponse::success_with_data(
        "结果文件添加成功",
        serde_json::json!({
            "clip_id": clip_id,
            "file_name": request.file_name
        })
    )))
}
