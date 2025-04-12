// API基础URL
const API_BASE_URL = '/api';

// DOM元素
const clipsList = document.getElementById('clipsList');
const noClips = document.getElementById('noClips');
const clipId = document.getElementById('clipId');
const statusClipId = document.getElementById('statusClipId');
const uploadForm = document.getElementById('uploadForm');
const updateStatusForm = document.getElementById('updateStatusForm');
const refreshBtn = document.getElementById('refreshBtn');
const clipDetailModal = new bootstrap.Modal(document.getElementById('clipDetailModal'));
const clipDetail = document.getElementById('clipDetail');
const resultFilesList = document.getElementById('resultFilesList');
const noResultFiles = document.getElementById('noResultFiles');

// 页面加载时获取任务列表
document.addEventListener('DOMContentLoaded', () => {
    fetchClips();
    
    // 绑定事件处理器
    refreshBtn.addEventListener('click', fetchClips);
    uploadForm.addEventListener('submit', handleUpload);
    updateStatusForm.addEventListener('submit', handleStatusUpdate);
});

// 获取所有剪辑任务
async function fetchClips() {
    try {
        showLoading(clipsList);
        
        const response = await fetch(`${API_BASE_URL}/clips`);
        if (!response.ok) {
            throw new Error('获取任务列表失败');
        }
        
        const data = await response.json();
        
        if (data.success && data.data && data.data.length > 0) {
            renderClipsList(data.data);
            updateClipSelects(data.data);
            noClips.classList.add('d-none');
        } else {
            clipsList.innerHTML = '';
            noClips.classList.remove('d-none');
            clipId.innerHTML = '<option value="">请选择任务</option>';
            statusClipId.innerHTML = '<option value="">请选择任务</option>';
        }
    } catch (error) {
        console.error('Error:', error);
        showError('获取任务列表失败: ' + error.message);
    } finally {
        hideLoading(clipsList);
    }
}

// 渲染任务列表
function renderClipsList(clips) {
    clipsList.innerHTML = '';
    
    clips.forEach(clip => {
        const row = document.createElement('tr');
        
        const statusClass = `status-${clip.status}`;
        const statusText = getStatusText(clip.status);
        
        row.innerHTML = `
            <td>${clip.id.substring(0, 8)}...</td>
            <td>${clip.prompt.length > 30 ? clip.prompt.substring(0, 30) + '...' : clip.prompt}</td>
            <td><span class="status-badge ${statusClass}">${statusText}</span></td>
            <td>${formatDate(clip.created_at)}</td>
            <td>
                <button class="btn btn-sm btn-outline-info view-btn" data-id="${clip.id}">查看</button>
                <button class="btn btn-sm btn-outline-primary upload-btn" data-id="${clip.id}">上传</button>
            </td>
        `;
        
        clipsList.appendChild(row);
    });
    
    // 绑定查看按钮事件
    document.querySelectorAll('.view-btn').forEach(btn => {
        btn.addEventListener('click', () => {
            const id = btn.getAttribute('data-id');
            viewClipDetail(id);
        });
    });
    
    // 绑定上传按钮事件
    document.querySelectorAll('.upload-btn').forEach(btn => {
        btn.addEventListener('click', () => {
            const id = btn.getAttribute('data-id');
            clipId.value = id;
            document.getElementById('resultFile').focus();
        });
    });
}

// 更新任务选择下拉框
function updateClipSelects(clips) {
    clipId.innerHTML = '<option value="">请选择任务</option>';
    statusClipId.innerHTML = '<option value="">请选择任务</option>';
    
    clips.forEach(clip => {
        const option = document.createElement('option');
        option.value = clip.id;
        option.textContent = `${clip.id.substring(0, 8)}... (${getStatusText(clip.status)})`;
        
        const statusOption = option.cloneNode(true);
        
        clipId.appendChild(option);
        statusClipId.appendChild(statusOption);
    });
}

// 查看任务详情
async function viewClipDetail(id) {
    try {
        const response = await fetch(`${API_BASE_URL}/clips/${id}`);
        if (!response.ok) {
            throw new Error('获取任务详情失败');
        }
        
        const data = await response.json();
        
        if (data.success && data.data) {
            const clip = data.data;
            
            // 渲染详情
            clipDetail.innerHTML = `
                <div class="row">
                    <div class="col-md-6">
                        <div class="detail-item">
                            <div class="detail-label">ID</div>
                            <div>${clip.id}</div>
                        </div>
                        <div class="detail-item">
                            <div class="detail-label">提示词</div>
                            <div>${clip.prompt}</div>
                        </div>
                        <div class="detail-item">
                            <div class="detail-label">状态</div>
                            <div><span class="status-badge status-${clip.status}">${getStatusText(clip.status)}</span></div>
                        </div>
                    </div>
                    <div class="col-md-6">
                        <div class="detail-item">
                            <div class="detail-label">素材路径</div>
                            <div>${clip.material_path}</div>
                        </div>
                        <div class="detail-item">
                            <div class="detail-label">输出路径</div>
                            <div>${clip.output_path}</div>
                        </div>
                        <div class="detail-item">
                            <div class="detail-label">创建时间</div>
                            <div>${formatDate(clip.created_at)}</div>
                        </div>
                        <div class="detail-item">
                            <div class="detail-label">更新时间</div>
                            <div>${formatDate(clip.updated_at)}</div>
                        </div>
                    </div>
                </div>
            `;
            
            // 渲染结果文件列表
            if (clip.result_files && clip.result_files.length > 0) {
                resultFilesList.innerHTML = '';
                
                clip.result_files.forEach(file => {
                    const li = document.createElement('li');
                    li.className = 'list-group-item file-item';
                    li.innerHTML = `
                        <i class="bi bi-file-earmark-play file-icon"></i>
                        <div class="file-name">${file}</div>
                        <div class="file-actions">
                            <a href="${API_BASE_URL}/download/${clip.id}?file=${encodeURIComponent(file)}" 
                               class="btn btn-sm btn-outline-primary" target="_blank">
                                下载
                            </a>
                        </div>
                    `;
                    resultFilesList.appendChild(li);
                });
                
                resultFilesList.classList.remove('d-none');
                noResultFiles.classList.add('d-none');
            } else {
                resultFilesList.innerHTML = '';
                resultFilesList.classList.add('d-none');
                noResultFiles.classList.remove('d-none');
            }
            
            // 显示模态框
            clipDetailModal.show();
        } else {
            showError('获取任务详情失败');
        }
    } catch (error) {
        console.error('Error:', error);
        showError('获取任务详情失败: ' + error.message);
    }
}

// 处理上传结果文件
async function handleUpload(event) {
    event.preventDefault();
    
    const id = clipId.value;
    const file = document.getElementById('resultFile').files[0];
    
    if (!id) {
        showError('请选择任务');
        return;
    }
    
    if (!file) {
        showError('请选择文件');
        return;
    }
    
    try {
        const formData = new FormData();
        formData.append('clip_id', id);
        formData.append('file', file);
        
        const response = await fetch(`${API_BASE_URL}/upload`, {
            method: 'POST',
            body: formData
        });
        
        const data = await response.json();
        
        if (data.success) {
            showSuccess('文件上传成功');
            uploadForm.reset();
            fetchClips();
        } else {
            showError(data.error || '上传失败');
        }
    } catch (error) {
        console.error('Error:', error);
        showError('上传失败: ' + error.message);
    }
}

// 处理更新任务状态
async function handleStatusUpdate(event) {
    event.preventDefault();
    
    const id = statusClipId.value;
    const status = document.getElementById('newStatus').value;
    
    if (!id) {
        showError('请选择任务');
        return;
    }
    
    if (!status) {
        showError('请选择状态');
        return;
    }
    
    try {
        const response = await fetch(`${API_BASE_URL}/clips/${id}/status`, {
            method: 'PUT',
            headers: {
                'Content-Type': 'application/json'
            },
            body: JSON.stringify({ status })
        });
        
        const data = await response.json();
        
        if (data.success) {
            showSuccess('状态更新成功');
            updateStatusForm.reset();
            fetchClips();
        } else {
            showError(data.error || '更新失败');
        }
    } catch (error) {
        console.error('Error:', error);
        showError('更新失败: ' + error.message);
    }
}

// 辅助函数
function getStatusText(status) {
    const statusMap = {
        'pending': '等待处理',
        'processing': '处理中',
        'completed': '已完成',
        'failed': '处理失败'
    };
    
    return statusMap[status] || status;
}

function formatDate(dateString) {
    const date = new Date(dateString);
    return date.toLocaleString('zh-CN');
}

function showLoading(element) {
    element.innerHTML = `
        <tr>
            <td colspan="5" class="text-center py-4">
                <div class="spinner-border text-primary" role="status">
                    <span class="visually-hidden">加载中...</span>
                </div>
                <div class="mt-2">加载中...</div>
            </td>
        </tr>
    `;
}

function hideLoading(element) {
    // 不需要做任何事情，因为内容会被新数据替换
}

function showError(message) {
    // 创建一个toast提示
    const toastContainer = document.createElement('div');
    toastContainer.className = 'position-fixed bottom-0 end-0 p-3';
    toastContainer.style.zIndex = '11';
    
    toastContainer.innerHTML = `
        <div class="toast align-items-center text-white bg-danger border-0" role="alert" aria-live="assertive" aria-atomic="true">
            <div class="d-flex">
                <div class="toast-body">
                    ${message}
                </div>
                <button type="button" class="btn-close btn-close-white me-2 m-auto" data-bs-dismiss="toast" aria-label="Close"></button>
            </div>
        </div>
    `;
    
    document.body.appendChild(toastContainer);
    
    const toastElement = toastContainer.querySelector('.toast');
    const toast = new bootstrap.Toast(toastElement, { delay: 5000 });
    toast.show();
    
    toastElement.addEventListener('hidden.bs.toast', () => {
        document.body.removeChild(toastContainer);
    });
}

function showSuccess(message) {
    // 创建一个toast提示
    const toastContainer = document.createElement('div');
    toastContainer.className = 'position-fixed bottom-0 end-0 p-3';
    toastContainer.style.zIndex = '11';
    
    toastContainer.innerHTML = `
        <div class="toast align-items-center text-white bg-success border-0" role="alert" aria-live="assertive" aria-atomic="true">
            <div class="d-flex">
                <div class="toast-body">
                    ${message}
                </div>
                <button type="button" class="btn-close btn-close-white me-2 m-auto" data-bs-dismiss="toast" aria-label="Close"></button>
            </div>
        </div>
    `;
    
    document.body.appendChild(toastContainer);
    
    const toastElement = toastContainer.querySelector('.toast');
    const toast = new bootstrap.Toast(toastElement, { delay: 3000 });
    toast.show();
    
    toastElement.addEventListener('hidden.bs.toast', () => {
        document.body.removeChild(toastContainer);
    });
}
