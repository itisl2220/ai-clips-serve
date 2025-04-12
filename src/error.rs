use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

/// API错误类型
#[derive(Debug, Error)]
pub enum ApiError {
    #[error("请求参数无效: {0}")]
    InvalidRequest(String),

    #[error("剪辑任务不存在: {0}")]
    ClipNotFound(String),

    #[error("文件不存在: {0}")]
    FileNotFound(String),

    #[error("文件操作失败: {0}")]
    FileOperationFailed(String),

    #[error("数据库操作失败: {0}")]
    DatabaseError(String),

    #[error("内部服务器错误: {0}")]
    InternalServerError(String),
}

impl ApiError {
    /// 获取对应的HTTP状态码
    pub fn status_code(&self) -> StatusCode {
        match self {
            Self::InvalidRequest(_) => StatusCode::BAD_REQUEST,
            Self::ClipNotFound(_) => StatusCode::NOT_FOUND,
            Self::FileNotFound(_) => StatusCode::NOT_FOUND,
            Self::FileOperationFailed(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::InternalServerError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let body = Json(json!({
            "success": false,
            "error": self.to_string(),
        }));

        (status, body).into_response()
    }
}

/// 结果类型别名
pub type Result<T> = std::result::Result<T, ApiError>;

/// 从各种错误类型转换为ApiError
impl From<std::io::Error> for ApiError {
    fn from(err: std::io::Error) -> Self {
        Self::FileOperationFailed(err.to_string())
    }
}

impl From<serde_json::Error> for ApiError {
    fn from(err: serde_json::Error) -> Self {
        Self::InvalidRequest(err.to_string())
    }
}

impl From<multer::Error> for ApiError {
    fn from(err: multer::Error) -> Self {
        Self::FileOperationFailed(err.to_string())
    }
}

impl From<anyhow::Error> for ApiError {
    fn from(err: anyhow::Error) -> Self {
        Self::InternalServerError(err.to_string())
    }
}
