# 文件上传功能使用文档

## 概述

本文档详细介绍了AI剪辑服务的文件上传功能，包括普通上传和分块上传两种方式。系统支持最大5GB的直接上传，对于更大的文件，建议使用分块上传功能。

## 1. 普通上传（适用于中小型文件）

### 接口信息

- **URL**: `/api/upload`
- **方法**: POST
- **Content-Type**: multipart/form-data
- **最大文件大小**: 5GB

### 请求参数

| 字段名 | 类型 | 必填 | 描述 |
|-------|------|-----|------|
| file  | File | 是  | 要上传的文件 |

### 响应示例

```json
{
  "code": 0,
  "message": "文件上传成功",
  "data": {
    "file_url": "/api/download/a1b2c3d4-e5f6-7890-abcd-ef1234567890.mp4",
    "file_name": "a1b2c3d4-e5f6-7890-abcd-ef1234567890.mp4",
    "original_file_name": "example.mp4"
  }
}
```

### 前端实现示例

```javascript
// 使用FormData上传文件
async function uploadFile(file) {
  const formData = new FormData();
  formData.append('file', file);
  
  try {
    const response = await fetch('/api/upload', {
      method: 'POST',
      body: formData,
    });
    
    const result = await response.json();
    return result;
  } catch (error) {
    console.error('文件上传失败:', error);
    throw error;
  }
}
```

## 2. 分块上传（适用于大文件）

分块上传将大文件分成多个小块，逐个上传，然后在服务器端合并。这种方式适用于大文件上传，可以提高上传成功率，支持断点续传，并提供上传进度信息。

### 步骤概述

1. 将文件分割成多个块
2. 上传第一个块，获取upload_id
3. 使用upload_id上传剩余的块
4. 所有块上传完成后，调用完成接口合并文件

### 2.1 上传分块

- **URL**: `/api/upload/chunk`
- **方法**: POST
- **Content-Type**: multipart/form-data

#### 请求参数

| 字段名 | 类型 | 必填 | 描述 |
|-------|------|-----|------|
| chunk_index | Number | 是 | 分块索引，从0开始 |
| total_chunks | Number | 是 | 总分块数 |
| upload_id | String | 否 | 上传ID，首次上传为空，后续上传必填 |
| original_file_name | String | 是 | 原始文件名 |
| file | File/Blob | 是 | 文件分块数据 |

#### 响应示例

```json
{
  "code": 0,
  "message": "success",
  "data": {
    "upload_id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
    "chunk_index": 0,
    "total_chunks": 10,
    "progress": 10.0
  }
}
```

### 2.2 完成分块上传

当所有分块都上传完成后，调用此接口合并文件。

- **URL**: `/api/upload/complete`
- **方法**: POST
- **Content-Type**: application/json

#### 请求参数

```json
{
  "upload_id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
  "original_file_name": "example.mp4"
}
```

#### 响应示例

```json
{
  "code": 0,
  "message": "文件上传成功",
  "data": {
    "file_url": "/api/download/b1c2d3e4-f5g6-7890-abcd-ef1234567890.mp4",
    "file_name": "b1c2d3e4-f5g6-7890-abcd-ef1234567890.mp4",
    "original_file_name": "example.mp4"
  }
}
```

### 前端实现示例

```javascript
// 分块上传实现
class ChunkedUploader {
  constructor(file, chunkSize = 5 * 1024 * 1024) { // 默认5MB一块
    this.file = file;
    this.chunkSize = chunkSize;
    this.totalChunks = Math.ceil(file.size / chunkSize);
    this.uploadId = null;
    this.currentChunk = 0;
    this.onProgress = null; // 进度回调函数
  }
  
  // 获取指定索引的分块
  getChunk(index) {
    const start = index * this.chunkSize;
    const end = Math.min(start + this.chunkSize, this.file.size);
    return this.file.slice(start, end);
  }
  
  // 上传单个分块
  async uploadChunk(index) {
    const chunk = this.getChunk(index);
    const formData = new FormData();
    
    formData.append('chunk_index', index);
    formData.append('total_chunks', this.totalChunks);
    formData.append('original_file_name', this.file.name);
    
    if (this.uploadId) {
      formData.append('upload_id', this.uploadId);
    }
    
    formData.append('file', chunk);
    
    const response = await fetch('/api/upload/chunk', {
      method: 'POST',
      body: formData,
    });
    
    const result = await response.json();
    
    if (result.code === 0) {
      this.uploadId = result.data.upload_id;
      if (this.onProgress) {
        this.onProgress(result.data.progress);
      }
      return result;
    } else {
      throw new Error(result.message || '上传分块失败');
    }
  }
  
  // 完成上传
  async complete() {
    if (!this.uploadId) {
      throw new Error('没有有效的上传ID');
    }
    
    const response = await fetch('/api/upload/complete', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        upload_id: this.uploadId,
        original_file_name: this.file.name,
      }),
    });
    
    return await response.json();
  }
  
  // 开始上传所有分块
  async uploadAll() {
    // 上传第一个分块获取uploadId
    await this.uploadChunk(0);
    this.currentChunk = 1;
    
    // 上传剩余分块
    const promises = [];
    for (let i = 1; i < this.totalChunks; i++) {
      promises.push(this.uploadChunk(i));
    }
    
    await Promise.all(promises);
    
    // 完成上传
    return await this.complete();
  }
  
  // 支持断点续传的上传方法
  async uploadWithResume() {
    try {
      // 上传所有分块
      for (let i = this.currentChunk; i < this.totalChunks; i++) {
        await this.uploadChunk(i);
        this.currentChunk++;
        
        // 可以在这里保存当前进度到localStorage，以便断点续传
        localStorage.setItem('uploadProgress_' + this.file.name, JSON.stringify({
          uploadId: this.uploadId,
          currentChunk: this.currentChunk,
          totalChunks: this.totalChunks
        }));
      }
      
      // 完成上传
      const result = await this.complete();
      
      // 清除进度记录
      localStorage.removeItem('uploadProgress_' + this.file.name);
      
      return result;
    } catch (error) {
      console.error('上传过程中出错:', error);
      throw error;
    }
  }
}

// 使用示例
async function uploadLargeFile(file) {
  const uploader = new ChunkedUploader(file);
  
  // 设置进度回调
  uploader.onProgress = (progress) => {
    console.log(`上传进度: ${progress.toFixed(2)}%`);
    // 更新UI进度条
  };
  
  try {
    // 检查是否有未完成的上传
    const savedProgress = localStorage.getItem('uploadProgress_' + file.name);
    if (savedProgress) {
      const progress = JSON.parse(savedProgress);
      uploader.uploadId = progress.uploadId;
      uploader.currentChunk = progress.currentChunk;
      console.log(`继续上传，从第${progress.currentChunk}块开始，共${progress.totalChunks}块`);
    }
    
    // 开始上传
    const result = await uploader.uploadWithResume();
    console.log('上传成功:', result);
    return result;
  } catch (error) {
    console.error('上传失败:', error);
    throw error;
  }
}
```

## 3. 文件下载

上传成功后，服务器会返回文件的访问URL，可以通过以下方式下载文件：

### 直接下载

- **URL**: 上传接口返回的file_url
- **方法**: GET

### 示例

```javascript
function downloadFile(fileUrl) {
  window.open(fileUrl, '_blank');
}
```

## 4. 最佳实践

1. **选择合适的上传方式**：
   - 小于100MB的文件：使用普通上传
   - 大于100MB的文件：使用分块上传

2. **分块大小建议**：
   - 稳定网络环境：5-10MB
   - 不稳定网络环境：2-5MB

3. **错误处理**：
   - 实现断点续传机制
   - 上传失败时自动重试
   - 保存上传进度到本地存储

4. **用户体验**：
   - 显示上传进度
   - 提供取消上传选项
   - 上传大文件时显示预计剩余时间

5. **安全考虑**：
   - 验证文件类型和大小
   - 限制允许上传的文件扩展名
   - 在客户端进行基本的文件完整性检查

## 5. 常见问题

### Q: 上传大文件时浏览器崩溃或内存不足
A: 使用分块上传功能，避免一次性加载整个文件到内存中。

### Q: 上传过程中网络中断
A: 使用断点续传功能，保存已上传的进度，网络恢复后继续上传。

### Q: 如何判断应该使用哪种上传方式
A: 根据文件大小自动选择：小于100MB使用普通上传，大于100MB使用分块上传。

### Q: 上传速度很慢
A: 可以尝试调整分块大小，网络稳定时使用更大的分块，网络不稳定时使用更小的分块。

---

如有任何问题，请联系技术支持团队。
