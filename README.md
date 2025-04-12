# AI混剪工具后端服务

这是一个基于Rust和Axum框架实现的AI混剪工具后端API服务。

## 功能特点

- 创建剪辑任务
- 上传素材文件
- 获取剪辑任务状态
- 获取剪辑任务详情
- 下载结果文件
- 支持PostgreSQL数据库存储剪辑任务信息

## 项目结构

```bash
ai-clips-serve/
├── src/
│   ├── models.rs     # 数据模型定义
│   ├── error.rs      # 错误处理
│   ├── services.rs   # 服务层实现
│   ├── routes.rs     # API路由和处理函数
│   ├── server.rs     # 服务器配置和启动
│   ├── db.rs         # 数据库连接和操作
│   └── main.rs       # 程序入口
├── scripts/
│   └── init_db.sql   # 数据库初始化脚本
├── .env              # 环境变量配置
├── Cargo.toml        # 项目依赖
└── README.md         # 项目说明
```

## 安装和运行

### 前置条件

- Rust 1.56.0 或更高版本
- Cargo 包管理器
- PostgreSQL 数据库服务器

### 数据库配置

在项目根目录创建 `.env` 文件，配置数据库连接信息：

```env
DB_HOST=数据库主机地址
DB_PORT=数据库端口
DB_USER=数据库用户名
DB_PASSWORD=数据库密码
DB_NAME=数据库名称
```

### 安装步骤

1. 克隆项目代码
2. 进入项目目录
3. 配置数据库连接信息（见上文）
4. 运行以下命令构建和启动服务

```bash
cargo build --release
cargo run --release
```

服务器默认运行在 `http://127.0.0.1:3000`

## API接口

### 1. 创建剪辑任务

- **路径**: `/api/clips`
- **方法**: POST
- **请求参数**:

```json
{
  "material_path": "素材文件夹路径",
  "output_path": "输出文件夹路径",
  "prompt": "AI剪辑提示词"
}
```

- **返回数据**: Clip对象（包含id等信息）

### 2. 上传素材文件

- **路径**: `/api/upload`
- **方法**: POST
- **请求参数**: multipart/form-data
  - `clip_id`: 剪辑任务ID
  - `file`: 文件数据
- **返回数据**: 成功状态

### 3. 获取剪辑任务状态

- **路径**: `/api/clips/{clip_id}/status`
- **方法**: GET
- **返回数据**: ClipStatus对象

### 4. 获取剪辑任务详情

- **路径**: `/api/clips/{clip_id}`
- **方法**: GET
- **返回数据**: Clip对象（包含结果文件列表等）

### 5. 下载结果文件

- **路径**: `/api/download/{clip_id}`
- **方法**: GET
- **请求参数**:
  - `file`: 文件名（查询参数）
- **返回数据**: 文件二进制数据
