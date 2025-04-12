// API基础URL
const API_BASE_URL = '/api';

// DOM元素
const clipsList = document.getElementById('clipsList');
const noClips = document.getElementById('noClips');
const refreshBtn = document.getElementById('refreshBtn');

// 模态框
const clipDetailModal = new bootstrap.Modal(document.getElementById('clipDetailModal'));
const uploadResultModal = new bootstrap.Modal(document.getElementById('uploadResultModal'));
const updateStatusModal = new bootstrap.Modal(document.getElementById('updateStatusModal'));

// 详情元素
const clipDetail = document.getElementById('clipDetail');
const resultFilesList = document.getElementById('resultFilesList');
const noResultFiles = document.getElementById('noResultFiles');

// 上传结果文件表单元素
const uploadResultForm = document.getElementById('uploadResultForm');
const uploadClipId = document.getElementById('uploadClipId');
const resultFiles = document.getElementById('resultFiles');
const uploadStatus = document.getElementById('uploadStatus');
const submitUpload = document.getElementById('submitUpload');

// 修改状态表单元素
const updateStatusForm = document.getElementById('updateStatusForm');
const statusClipId = document.getElementById('statusClipId');
const newStatus = document.getElementById('newStatus');
const submitStatus = document.getElementById('submitStatus');

// 页面加载时获取任务列表
document.addEventListener('DOMContentLoaded', () => {
    fetchClips();
    
    // 绑定事件处理器
    refreshBtn.addEventListener('click', fetchClips);
    submitUpload.addEventListener('click', handleUploadResult);
    submitStatus.addEventListener('click', handleStatusUpdate);
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
            noClips.classList.add('d-none');
        } else {
            clipsList.innerHTML = '';
            noClips.classList.remove('d-none');
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
                <button class="btn btn-sm btn-outline-primary upload-btn" data-id="${clip.id}">上传结果</button>
                <button class="btn btn-sm btn-outline-secondary status-btn" data-id="${clip.id}">修改状态</button>
                ${clip.material_file ? `<button class="btn btn-sm btn-outline-success download-material-btn" data-url="${clip.material_file}">下载素材</button>` : ''}
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
    
    // 绑定上传结果按钮事件
    document.querySelectorAll('.upload-btn').forEach(btn => {
        btn.addEventListener('click', () => {
            const id = btn.getAttribute('data-id');
            // 确保元素存在
            if (uploadClipId && resultFiles && uploadStatus) {
                uploadClipId.value = id;
                resultFiles.value = '';
                uploadStatus.value = '';
                uploadResultModal.show();
            } else {
                console.error('上传结果文件所需元素不存在');
                showError('上传结果文件功能初始化失败，请刷新页面重试');
            }
        });
    });
    
    // 绑定修改状态按钮事件
    document.querySelectorAll('.status-btn').forEach(btn => {
        btn.addEventListener('click', () => {
            const id = btn.getAttribute('data-id');
            // 确保元素存在
            if (statusClipId && newStatus) {
                statusClipId.value = id;
                newStatus.value = '';
                updateStatusModal.show();
            } else {
                console.error('修改状态所需元素不存在');
                showError('修改状态功能初始化失败，请刷新页面重试');
            }
        });
    });
    
    // 绑定下载素材按钮事件
    document.querySelectorAll('.download-material-btn').forEach(btn => {
        btn.addEventListener('click', () => {
            const url = btn.getAttribute('data-url');
            if (url) {
                // 创建一个临时链接并模拟点击
                const a = document.createElement('a');
                a.href = url;
                a.target = '_blank';
                a.rel = 'noopener noreferrer';
                document.body.appendChild(a);
                a.click();
                document.body.removeChild(a);
            } else {
                showError('素材链接无效');
            }
        });
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
                        ${clip.material_file ? `
                        <div class="detail-item">
                            <div class="detail-label">素材包</div>
                            <div><a href="${clip.material_file}" target="_blank">下载素材包</a></div>
                        </div>
                        ` : ''}
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
                        <div class="file-name">${file}</div>
                        <div class="file-actions">
                            <a href="${API_BASE_URL}/download/${clip.id}/file?name=${encodeURIComponent(file)}" 
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
async function handleUploadResult() {
    const id = uploadClipId.value;
    const files = resultFiles.files;
    const status = uploadStatus.value;
    
    if (!id) {
        showError('任务ID无效');
        return;
    }
    
    if (files.length === 0) {
        showError('请选择至少一个文件');
        return;
    }
    
    console.log(`准备上传 ${files.length} 个文件到任务 ${id}`);
    
    try {
        // 上传所有文件
        const uploadPromises = Array.from(files).map(async (file, index) => {
            console.log(`开始上传第 ${index + 1}/${files.length} 个文件: ${file.name}`);
            
            const formData = new FormData();
            formData.append('file', file);
            
            const response = await fetch(`${API_BASE_URL}/upload`, {
                method: 'POST',
                body: formData
            });
            
            if (!response.ok) {
                throw new Error(`上传文件 ${file.name} 失败，状态码: ${response.status}`);
            }
            
            const data = await response.json();
            
            if (!data.success) {
                throw new Error(data.error || `上传文件 ${file.name} 失败`);
            }
            
            console.log(`文件 ${file.name} 上传成功，获取到链接: ${data.data.file_url}`);
            
            // 添加结果文件到任务
            const fileUrl = data.data.file_url;
            const fileName = data.data.file_name;
            
            console.log(`将文件 ${fileName} 添加到任务 ${id} 的结果文件列表`);
            
            const resultResponse = await fetch(`${API_BASE_URL}/clips/${id}/result`, {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify({ file_name: fileName })
            });
            
            if (!resultResponse.ok) {
                throw new Error(`添加结果文件 ${fileName} 失败，状态码: ${resultResponse.status}`);
            }
            
            const resultData = await resultResponse.json();
            
            if (!resultData.success) {
                throw new Error(resultData.error || `添加结果文件 ${fileName} 失败`);
            }
            
            console.log(`文件 ${fileName} 成功添加到任务 ${id} 的结果文件列表`);
            
            return { fileName, fileUrl };
        });
        
        const results = await Promise.all(uploadPromises);
        console.log(`所有 ${results.length} 个文件上传并添加到任务成功`);
        
        // 如果选择了状态，则更新任务状态
        if (status) {
            console.log(`更新任务 ${id} 的状态为 ${status}`);
            
            const statusResponse = await fetch(`${API_BASE_URL}/clips/${id}/status`, {
                method: 'PUT',
                headers: {
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify({ status })
            });
            
            if (!statusResponse.ok) {
                console.warn(`更新任务状态失败，状态码: ${statusResponse.status}`);
            } else {
                const statusData = await statusResponse.json();
                if (!statusData.success) {
                    console.warn(`更新任务状态失败: ${statusData.error || '未知错误'}`);
                } else {
                    console.log(`任务状态更新成功`);
                }
            }
        }
        
        showSuccess(`成功上传 ${results.length} 个结果文件`);
        uploadResultModal.hide();
        
        // 刷新任务列表并自动打开任务详情
        await fetchClips();
        viewClipDetail(id);
    } catch (error) {
        console.error('Error:', error);
        showError('上传失败: ' + error.message);
    }
}

// 处理更新任务状态
async function handleStatusUpdate() {
    const id = statusClipId.value;
    const status = newStatus.value;
    
    if (!id) {
        showError('请选择任务');
        return;
    }
    
    if (!status) {
        showError('请选择状态');
        return;
    }
    
    // 如果要将状态设置为已完成，需要验证任务是否有结果文件
    if (status === 'completed') {
        try {
            const response = await fetch(`${API_BASE_URL}/clips/${id}`);
            if (!response.ok) {
                throw new Error('获取任务详情失败');
            }
            
            const data = await response.json();
            
            if (data.success && data.data) {
                const clip = data.data;
                if (!clip.result_files || clip.result_files.length === 0) {
                    showError('没有结果文件的任务不能标记为已完成');
                    return;
                }
            }
        } catch (error) {
            console.error('Error:', error);
            showError('验证任务状态失败: ' + error.message);
            return;
        }
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
            updateStatusModal.hide();
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
    switch(status) {
        case 'pending': return '等待处理';
        case 'processing': return '处理中';
        case 'completed': return '已完成';
        case 'failed': return '处理失败';
        default: return status;
    }
}

function formatDate(dateString) {
    return new Date(dateString).toLocaleString('zh-CN');
}

function showLoading(element) {
    if (!element) return; // 防止 element 为 null
    
    // 创建加载指示器
    const loadingDiv = document.createElement('div');
    loadingDiv.className = 'loading-indicator';
    loadingDiv.innerHTML = `
        <div class="spinner-border text-primary" role="status">
            <span class="visually-hidden">加载中...</span>
        </div>
        <div class="mt-2">加载中...</div>
    `;
    
    // 添加到元素
    if (element.parentNode) {
        element.parentNode.insertBefore(loadingDiv, element);
    } else {
        console.warn('无法添加加载指示器：元素没有父节点');
    }
}

function hideLoading(element) {
    if (!element || !element.parentNode) return; // 防止 element 为 null
    
    // 移除加载指示器
    const loadingIndicator = element.parentNode.querySelector('.loading-indicator');
    if (loadingIndicator) {
        loadingIndicator.remove();
    }
}

function showError(message) {
    // 创建错误提示
    const alertDiv = document.createElement('div');
    alertDiv.className = 'alert alert-danger alert-dismissible fade show position-fixed top-0 start-50 translate-middle-x mt-3';
    alertDiv.setAttribute('role', 'alert');
    alertDiv.style.zIndex = '9999';
    alertDiv.innerHTML = `
        ${message}
        <button type="button" class="btn-close" data-bs-dismiss="alert" aria-label="Close"></button>
    `;
    
    // 添加到页面
    document.body.appendChild(alertDiv);
    
    // 5秒后自动关闭
    setTimeout(() => {
        const bsAlert = new bootstrap.Alert(alertDiv);
        bsAlert.close();
    }, 5000);
}

function showSuccess(message) {
    // 创建成功提示
    const alertDiv = document.createElement('div');
    alertDiv.className = 'alert alert-success alert-dismissible fade show position-fixed top-0 start-50 translate-middle-x mt-3';
    alertDiv.setAttribute('role', 'alert');
    alertDiv.style.zIndex = '9999';
    alertDiv.innerHTML = `
        ${message}
        <button type="button" class="btn-close" data-bs-dismiss="alert" aria-label="Close"></button>
    `;
    
    // 添加到页面
    document.body.appendChild(alertDiv);
    
    // 3秒后自动关闭
    setTimeout(() => {
        const bsAlert = new bootstrap.Alert(alertDiv);
        bsAlert.close();
    }, 3000);
}
