use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use sqlx::{postgres::PgRow, Row};

/// 剪辑状态枚举
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ClipStatus {
    Pending,     // 等待处理
    Processing,  // 处理中
    Completed,   // 已完成
    Failed,      // 处理失败
}

impl Default for ClipStatus {
    fn default() -> Self {
        Self::Pending
    }
}

impl std::fmt::Display for ClipStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ClipStatus::Pending => write!(f, "pending"),
            ClipStatus::Processing => write!(f, "processing"),
            ClipStatus::Completed => write!(f, "completed"),
            ClipStatus::Failed => write!(f, "failed"),
        }
    }
}

impl From<&str> for ClipStatus {
    fn from(s: &str) -> Self {
        // 转换为小写并去除空白字符，提高容错性
        let s = s.trim().to_lowercase();
        match s.as_str() {
            "pending" => ClipStatus::Pending,
            "processing" => ClipStatus::Processing,
            "completed" => ClipStatus::Completed,
            "failed" => ClipStatus::Failed,
            _ => ClipStatus::Pending,
        }
    }
}

/// 剪辑任务
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Clip {
    pub id: String,                    // 剪辑任务ID
    pub material_path: String,         // 素材文件夹路径
    pub output_path: String,           // 输出文件夹路径
    pub prompt: String,                // AI剪辑提示词
    pub status: ClipStatus,            // 剪辑状态
    pub created_at: DateTime<Utc>,     // 创建时间
    pub updated_at: DateTime<Utc>,     // 更新时间
    pub result_files: Vec<String>,     // 结果文件列表
    pub material_file: Option<String>, // 素材包链接
}

impl Clip {
    /// 创建新的剪辑任务
    pub fn new(material_path: String, output_path: String, prompt: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            material_path,
            output_path,
            prompt,
            status: ClipStatus::default(),
            created_at: now,
            updated_at: now,
            result_files: Vec::new(),
            material_file: None,
        }
    }

    /// 更新剪辑任务状态
    pub fn update_status(&mut self, status: ClipStatus) {
        self.status = status;
        self.updated_at = Utc::now();
    }

    /// 添加结果文件
    pub fn add_result_file(&mut self, file_path: String) {
        self.result_files.push(file_path);
        self.updated_at = Utc::now();
    }
    
    /// 从数据库行创建剪辑任务
    pub fn from_row(row: PgRow) -> Result<Self, sqlx::Error> {
        let result_files: serde_json::Value = row.try_get("result_files")?;
        let result_files: Vec<String> = serde_json::from_value(result_files)
            .unwrap_or_else(|_| Vec::new());
            
        // 安全地获取 material_file 字段，如果不存在则返回 None
        let material_file: Option<String> = match row.try_get::<Option<serde_json::Value>, _>("material_file") {
            Ok(Some(value)) => serde_json::from_value(value).unwrap_or(None),
            _ => None,
        };
            
        Ok(Self {
            id: row.try_get("id")?,
            material_path: row.try_get("material_path")?,
            output_path: row.try_get("output_path")?,
            prompt: row.try_get("prompt")?,
            status: ClipStatus::from(row.try_get::<&str, _>("status")?),
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
            result_files,
            material_file,
        })
    }
}

/// 创建剪辑任务请求
#[derive(Debug, Deserialize)]
pub struct CreateClipRequest {
    pub material_path: String,
    pub output_path: String,
    pub prompt: String,
}

/// 上传文件请求
#[derive(Debug, Deserialize)]
pub struct UploadFileRequest {
    pub clip_id: String,
    pub file: Vec<u8>,
}

/// 更新素材链接请求
#[derive(Debug, Deserialize)]
pub struct UpdateMaterialFileRequest {
    pub material_file: String,
}

/// API响应包装器
#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

impl<T> ApiResponse<T> {
    /// 创建成功响应
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    /// 创建带数据和消息的成功响应
    pub fn success_with_data(message: impl Into<String>, data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: Some(message.into()),
        }
    }

    /// 创建错误响应
    pub fn error(message: impl Into<String>) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message.into()),
        }
    }
}
