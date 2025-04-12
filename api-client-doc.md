# AI混剪工具客户端接口列表和数据结构

## 接口列表

### 1. 健康检查
- **URL**: `/api/health`
- **方法**: `GET`
- **响应**:
  ```json
  {
    "success": true,
    "data": "服务正常"
  }
  ```

### 2. 创建剪辑任务
- **URL**: `/api/clips`
- **方法**: `POST`
- **请求体**:
  ```json
  {
    "material_path": "素材文件夹路径",
    "output_path": "输出文件夹路径",
    "prompt": "AI剪辑提示词"
  }
  ```
- **响应**:
  ```json
  {
    "success": true,
    "data": {
      "id": "任务ID",
      "material_path": "素材文件夹路径",
      "output_path": "输出文件夹路径",
      "prompt": "AI剪辑提示词",
      "status": "pending",
      "created_at": "创建时间",
      "updated_at": "更新时间",
      "result_files": []
    }
  }
  ```

### 3. 获取所有剪辑任务
- **URL**: `/api/clips`
- **方法**: `GET`
- **响应**:
  ```json
  {
    "success": true,
    "data": [
      {
        "id": "任务ID",
        "material_path": "素材文件夹路径",
        "output_path": "输出文件夹路径",
        "prompt": "AI剪辑提示词",
        "status": "pending",
        "created_at": "创建时间",
        "updated_at": "更新时间",
        "result_files": []
      }
    ]
  }
  ```

### 4. 获取剪辑任务详情
- **URL**: `/api/clips/{clip_id}`
- **方法**: `GET`
- **响应**:
  ```json
  {
    "success": true,
    "data": {
      "id": "任务ID",
      "material_path": "素材文件夹路径",
      "output_path": "输出文件夹路径",
      "prompt": "AI剪辑提示词",
      "status": "pending",
      "created_at": "创建时间",
      "updated_at": "更新时间",
      "result_files": ["文件1.mp4", "文件2.mp4"]
    }
  }
  ```

### 5. 获取剪辑任务状态
- **URL**: `/api/clips/{clip_id}/status`
- **方法**: `GET`
- **响应**:
  ```json
  {
    "success": true,
    "data": "pending"
  }
  ```

### 6. 更新剪辑任务状态
- **URL**: `/api/clips/{clip_id}/status`
- **方法**: `PUT`
- **请求体**:
  ```json
  {
    "status": "completed"
  }
  ```
- **响应**:
  ```json
  {
    "success": true,
    "data": "状态更新成功"
  }
  ```

### 7. 上传结果文件
- **URL**: `/api/upload`
- **方法**: `POST`
- **请求体**: `multipart/form-data`
  - `clip_id`: 剪辑任务ID
  - `file`: 文件数据
- **响应**:
  ```json
  {
    "success": true,
    "data": "文件上传成功"
  }
  ```

### 8. 下载结果文件
- **URL**: `/api/download/{clip_id}?file=文件名.mp4`
- **方法**: `GET`
- **响应**: 文件内容（二进制流）

## 数据结构

### 1. 剪辑任务 (Clip)
```typescript
interface Clip {
  id: string;                  // 剪辑任务ID
  material_path: string;       // 素材文件夹路径
  output_path: string;         // 输出文件夹路径
  prompt: string;              // AI剪辑提示词
  status: ClipStatus;          // 剪辑状态
  created_at: string;          // 创建时间
  updated_at: string;          // 更新时间
  result_files: string[];      // 结果文件列表
}
```

### 2. 剪辑状态 (ClipStatus)
```typescript
type ClipStatus = "pending" | "processing" | "completed" | "failed";
```

### 3. 创建剪辑任务请求 (CreateClipRequest)
```typescript
interface CreateClipRequest {
  material_path: string;       // 素材文件夹路径
  output_path: string;         // 输出文件夹路径
  prompt: string;              // AI剪辑提示词
}
```

### 4. 更新状态请求 (UpdateStatusRequest)
```typescript
interface UpdateStatusRequest {
  status: ClipStatus;          // 新状态
}
```

### 5. API响应 (ApiResponse)
```typescript
interface ApiResponse<T> {
  success: boolean;            // 是否成功
  data?: T;                    // 响应数据
  error?: string;              // 错误信息
}
```

## 客户端示例代码

### 创建剪辑任务
```javascript
async function createClip(materialPath, outputPath, prompt) {
  try {
    const response = await fetch('/api/clips', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json'
      },
      body: JSON.stringify({
        material_path: materialPath,
        output_path: outputPath,
        prompt: prompt
      })
    });
    
    return await response.json();
  } catch (error) {
    console.error('创建剪辑任务失败:', error);
    throw error;
  }
}
```

### 上传结果文件
```javascript
async function uploadResultFile(clipId, file) {
  try {
    const formData = new FormData();
    formData.append('clip_id', clipId);
    formData.append('file', file);
    
    const response = await fetch('/api/upload', {
      method: 'POST',
      body: formData
    });
    
    return await response.json();
  } catch (error) {
    console.error('上传文件失败:', error);
    throw error;
  }
}
```

### 更新任务状态
```javascript
async function updateClipStatus(clipId, status) {
  try {
    const response = await fetch(`/api/clips/${clipId}/status`, {
      method: 'PUT',
      headers: {
        'Content-Type': 'application/json'
      },
      body: JSON.stringify({ status })
    });
    
    return await response.json();
  } catch (error) {
    console.error('更新状态失败:', error);
    throw error;
  }
}
```
