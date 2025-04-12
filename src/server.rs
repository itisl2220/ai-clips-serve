use std::{net::SocketAddr, path::PathBuf, sync::Arc};

use axum::{Router, Server};
use tower_http::{
    cors::{Any, CorsLayer},
    services::ServeDir,
    trace::TraceLayer,
};

use crate::{
    routes::create_router, 
    services::ClipService,
    db::{DbConfig, create_pool, init_db},
};

/// 服务器配置
#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub addr: SocketAddr,
    pub base_dir: PathBuf,
    pub db_config: DbConfig,
    pub static_dir: PathBuf,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            addr: "127.0.0.1:8080".parse().unwrap(),
            base_dir: PathBuf::from("./data"),
            db_config: DbConfig {
                host: "localhost".to_string(),
                port: 5432,
                username: "postgres".to_string(),
                password: "postgres".to_string(),
                database: "ai_clips".to_string(),
            },
            static_dir: PathBuf::from("./static"),
        }
    }
}

/// API服务器
pub struct ApiServer {
    config: ServerConfig,
}

impl ApiServer {
    /// 创建新的API服务器实例
    pub fn new(config: ServerConfig) -> Self {
        Self { config }
    }

    /// 启动服务器
    pub async fn run(&self) -> anyhow::Result<()> {
        // 初始化日志 - 使用简单配置
        tracing_subscriber::fmt::init();
        
        println!("日志系统初始化完成");
        println!("服务器配置: {:?}", self.config);

        // 连接数据库
        println!("正在连接数据库...");
        let pool = create_pool(&self.config.db_config).await?;
        
        // 初始化数据库表
        println!("正在初始化数据库表...");
        init_db(&pool).await?;

        // 创建剪辑服务
        let clip_service = Arc::new(ClipService::new(pool, &self.config.base_dir));

        // 创建CORS中间件
        let cors = CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any);

        // 创建静态文件服务
        let static_service = ServeDir::new(&self.config.static_dir);

        // 创建API路由
        let api_routes = create_router(clip_service.clone());

        // 创建应用
        let app = Router::new()
            .nest("/api", api_routes)
            .nest_service("/static", static_service)
            .layer(TraceLayer::new_for_http())
            .layer(cors);

        // 启动服务器
        println!("服务器启动在 {}", self.config.addr);
        Server::bind(&self.config.addr)
            .serve(app.into_make_service())
            .await?;

        Ok(())
    }
}
