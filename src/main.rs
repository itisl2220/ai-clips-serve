mod db;
mod error;
mod models;
mod routes;
mod server;
mod services;

use std::path::PathBuf;
use dotenv::dotenv;
use std::env;

use server::{ApiServer, ServerConfig};
use db::DbConfig;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 加载环境变量
    dotenv().ok();
    
    // 创建数据目录
    let base_dir = PathBuf::from("./data");
    std::fs::create_dir_all(&base_dir)?;

    // 创建数据库配置
    let db_config = DbConfig {
        host: env::var("DB_HOST").unwrap_or_else(|_| "pgm-uf6zc7jb68b028mmso.rwlb.rds.aliyuncs.com".to_string()),
        port: env::var("DB_PORT").unwrap_or_else(|_| "5432".to_string()).parse().unwrap_or(5432),
        username: env::var("DB_USER").unwrap_or_else(|_| "liuzhonyu".to_string()),
        password: env::var("DB_PASSWORD").unwrap_or_else(|_| "1997520liuzhonyU@".to_string()),
        database: env::var("DB_NAME").unwrap_or_else(|_| "clai".to_string()),
    };

    // 创建服务器配置
    let config = ServerConfig {
        addr: "127.0.0.1:8080".parse().unwrap(),
        base_dir,
        db_config,
        static_dir: PathBuf::from("./static"),
    };

    // 创建并启动服务器
    let server = ApiServer::new(config);
    server.run().await?;

    Ok(())
}
