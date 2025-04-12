use std::sync::Arc;

use axum::{
    extract::{Multipart, Path, Query},
    http::{header, StatusCode},
    response::IntoResponse,
    routing::{get, post, put},
    Extension, Json, Router,
};
use serde::Deserialize;
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
        .route("/upload", post(upload_file))
        .route("/clips/:clip_id/status", get(get_clip_status))
        .route("/clips/:clip_id/status", put(update_clip_status))
        .route("/clips/:clip_id", get(get_clip))
        .route("/clips/:clip_id/material", put(update_material_file))
        .route("/download/:clip_id/file", get(download_file))
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
    
    // 启动异步处理任务
    let service_clone = clip_service.clone();
    let clip_id = clip.id.clone();
    tokio::spawn(async move {
        if let Err(e) = service_clone.process_clip(&clip_id).await {
            eprintln!("处理剪辑任务失败: {}", e);
            let _ = service_clone.update_clip_status(&clip_id, ClipStatus::Failed).await;
        }
    });
    
    Ok(Json(ApiResponse::success(clip)))
}

/// 获取所有剪辑任务
async fn get_all_clips(
    Extension(clip_service): Extension<Arc<ClipService>>,
) -> Result<impl IntoResponse> {
    let clips = clip_service.get_all_clips().await?;
    Ok(Json(ApiResponse::success(clips)))
}

/// 上传素材文件
async fn upload_file(
    Extension(clip_service): Extension<Arc<ClipService>>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse> {
    println!("开始处理文件上传请求");
    let mut clip_id = String::new();
    let mut original_file_name = String::new();
    let mut file_data = Vec::new();
    

    // 解析multipart表单数据
    println!("开始解析multipart表单数据");
    while let Some(field) = multipart.next_field().await.map_err(|e| {
        println!("解析表单数据失败: {}", e);
        ApiError::InvalidRequest(format!("解析表单数据失败: {}", e))
    })? {
        let name = field.name().unwrap_or("").to_string();
        println!("处理表单字段: {}", name);
        if name == "clip_id" {
            println!("读取clip_id字段");
            clip_id = field.text().await.map_err(|e| {
                println!("读取clip_id失败: {}", e);
                ApiError::InvalidRequest(format!("读取clip_id失败: {}", e))
            })?;
            println!("clip_id: {}", clip_id);
        } else if name == "file" {
            println!("读取file字段");
            let field_file_name = field.file_name().unwrap_or("unknown").to_string();
            println!("原始文件名: {}", field_file_name);
            original_file_name = field_file_name;
            
            println!("读取文件数据");
            let bytes = field.bytes().await.map_err(|e| {
                println!("读取文件数据失败: {}", e);
                ApiError::FileOperationFailed(format!("读取文件数据失败: {}", e))
            })?;
            println!("文件大小: {} 字节", bytes.len());
            file_data = bytes.to_vec();
        } else {
            println!("忽略未知字段: {}", name);
        }
    }
    
    // 验证必要参数
    println!("验证必要参数");
    if clip_id.is_empty() {
        println!("缺少clip_id参数");
        return Err(ApiError::InvalidRequest("缺少clip_id参数".to_string()));
    }
    
    if original_file_name.is_empty() || file_data.is_empty() {
        println!("缺少文件数据");
        return Err(ApiError::InvalidRequest("缺少文件数据".to_string()));
    }
    
    // 生成UUID作为文件名，但保留原始扩展名
    let extension = original_file_name.split('.').last().unwrap_or("");
    let uuid_file_name = if !extension.is_empty() {
        format!("{}.{}", Uuid::new_v4(), extension)
    } else {
        Uuid::new_v4().to_string()
    };
    
    println!("生成UUID文件名: {}", uuid_file_name);
    
    // 上传文件并获取文件链接
    println!("开始上传文件: clip_id={}, uuid_file_name={}, 文件大小={}字节", clip_id, uuid_file_name, file_data.len());
    let file_url = clip_service.upload_file(&clip_id, &uuid_file_name, file_data).await?;
    println!("文件上传成功，链接: {}", file_url);
    
    // 返回成功响应，包含文件链接
    println!("返回成功响应");
    Ok(Json(ApiResponse::success_with_data(
        "文件上传成功",
        serde_json::json!({
            "file_url": file_url,
            "clip_id": clip_id,
            "file_name": uuid_file_name,
            "original_file_name": original_file_name
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
