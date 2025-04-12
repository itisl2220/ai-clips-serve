# AI混剪工具后端API规范

## 接口列表

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

## 数据结构

### 1. Clip（剪辑任务）
```json
{
  "id": "String",           // 剪辑任务ID
  "material_path": "String", // 素材文件夹路径
  "output_path": "String",   // 输出文件夹路径
  "prompt": "String",        // AI剪辑提示词
  "status": "ClipStatus",    // 剪辑状态（枚举值）
  "created_at": "DateTime",  // 创建时间
  "updated_at": "DateTime",  // 更新时间
  "result_files": ["String"] // 结果文件列表
}
```

### 2. ClipStatus（剪辑状态枚举）
```json
enum ClipStatus {
  "pending",     // 等待处理
  "processing",  // 处理中
  "completed",   // 已完成
  "failed"       // 处理失败
}
```

## 后端处理流程
1. 接收创建剪辑任务请求，生成唯一ID
2. 接收并存储上传的素材文件
3. 根据提示词使用AI进行视频混剪处理
4. 提供状态查询接口供前端轮询
5. 处理完成后提供结果文件下载

## 建议实现细节
1. 添加任务队列管理多个剪辑请求
2. 实现进度详情反馈（当前只有状态，没有具体进度百分比）
3. 添加错误处理和日志记录
4. 考虑添加用户认证和授权机制
5. 实现文件存储管理（可考虑使用对象存储服务）
6. 添加任务超时处理机制
