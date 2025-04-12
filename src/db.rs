use sqlx::{postgres::PgPoolOptions, PgPool};
use std::env;

/// 数据库配置
#[derive(Debug, Clone)]
pub struct DbConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub database: String,
}

impl DbConfig {
    /// 创建数据库连接字符串
    pub fn connection_string(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username, self.password, self.host, self.port, self.database
        )
    }
}

/// 创建数据库连接池
pub async fn create_pool(config: &DbConfig) -> Result<PgPool, sqlx::Error> {
    let connection_string = config.connection_string();
    
    PgPoolOptions::new()
        .max_connections(10)
        .connect(&connection_string)
        .await
}

/// 初始化数据库
pub async fn init_db(pool: &PgPool) -> Result<(), sqlx::Error> {
    // 创建剪辑任务表
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS clips (
            id VARCHAR(36) PRIMARY KEY,
            material_path TEXT NOT NULL,
            output_path TEXT NOT NULL,
            prompt TEXT NOT NULL,
            status VARCHAR(20) NOT NULL,
            created_at TIMESTAMP WITH TIME ZONE NOT NULL,
            updated_at TIMESTAMP WITH TIME ZONE NOT NULL,
            result_files JSONB NOT NULL DEFAULT '[]'::JSONB,
            material_file JSONB DEFAULT NULL
        )
        "#,
    )
    .execute(pool)
    .await?;

    Ok(())
}
