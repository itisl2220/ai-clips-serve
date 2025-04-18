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
            
            // 显示提示词内容
            const promptContent = document.getElementById('promptContent');
            if (promptContent) {
                promptContent.textContent = clip.prompt;
            }
            
            // 设置复制按钮功能
            const copyPromptBtn = document.getElementById('copyPromptBtn');
            const copyFeedback = document.getElementById('copyFeedback');
            
            if (copyPromptBtn) {
                // 移除之前的事件监听器
                const newCopyBtn = copyPromptBtn.cloneNode(true);
                copyPromptBtn.parentNode.replaceChild(newCopyBtn, copyPromptBtn);
                
                // 添加新的事件监听器
                newCopyBtn.addEventListener('click', () => {
                    // 复制提示词到剪贴板
                    navigator.clipboard.writeText(clip.prompt)
                        .then(() => {
                            // 显示成功反馈
                            copyFeedback.textContent = '已复制!';
                            copyFeedback.classList.remove('d-none');
                            
                            // 更改按钮样式
                            newCopyBtn.classList.remove('btn-outline-primary');
                            newCopyBtn.classList.add('btn-success');
                            newCopyBtn.innerHTML = '<i class="bi bi-check"></i> 已复制';
                            
                            // 2秒后恢复按钮状态
                            setTimeout(() => {
                                copyFeedback.classList.add('d-none');
                                newCopyBtn.classList.remove('btn-success');
                                newCopyBtn.classList.add('btn-outline-primary');
                                newCopyBtn.innerHTML = '<i class="bi bi-clipboard"></i> 复制';
                            }, 2000);
                        })
                        .catch(err => {
                            console.error('复制失败:', err);
                            copyFeedback.textContent = '复制失败!';
                            copyFeedback.classList.remove('d-none');
                            copyFeedback.classList.remove('text-success');
                            copyFeedback.classList.add('text-danger');
                            
                            // 2秒后隐藏错误信息
                            setTimeout(() => {
                                copyFeedback.classList.add('d-none');
                                copyFeedback.classList.remove('text-danger');
                                copyFeedback.classList.add('text-success');
                            }, 2000);
                        });
                });
            }
            
            // 初始化工具提示
            const tooltipTriggerList = [].slice.call(document.querySelectorAll('[data-bs-toggle="tooltip"]'));
            tooltipTriggerList.map(function (tooltipTriggerEl) {
                return new bootstrap.Tooltip(tooltipTriggerEl);
            });
            
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
    
    // 获取进度显示元素
    const progressContainer = document.getElementById('uploadProgressContainer');
    const progressBar = document.getElementById('uploadProgressBar');
    const statusText = document.getElementById('uploadStatusText');
    const detailText = document.getElementById('uploadDetailText');
    
    // 显示进度条
    progressContainer.classList.remove('d-none');
    progressBar.style.width = '0%';
    progressBar.textContent = '0%';
    progressBar.setAttribute('aria-valuenow', 0);
    statusText.textContent = '准备上传...';
    detailText.textContent = '';
    
    // 禁用提交按钮
    submitUpload.disabled = true;
    
    try {
        // 计算每个文件的进度权重
        const totalFiles = files.length;
        const fileWeight = 90 / totalFiles; // 留出10%用于状态更新
        let totalProgress = 0;
        
        // 更新进度函数
        const updateProgress = (fileIndex, fileProgress, message) => {
            // 计算总体进度
            const fileBaseProgress = fileIndex * fileWeight;
            const currentFileProgress = fileProgress * fileWeight / 100;
            totalProgress = Math.min(90, fileBaseProgress + currentFileProgress);
            
            // 更新进度条
            const displayProgress = Math.round(totalProgress);
            progressBar.style.width = `${displayProgress}%`;
            progressBar.textContent = `${displayProgress}%`;
            progressBar.setAttribute('aria-valuenow', displayProgress);
            
            // 更新状态文本
            if (message) {
                statusText.textContent = message;
            }
            
            // 更新详细信息
            detailText.textContent = `处理第 ${fileIndex + 1}/${totalFiles} 个文件`;
        };
        
        // 上传所有文件
        const results = [];
        for (let i = 0; i < files.length; i++) {
            const file = files[i];
            updateProgress(i, 0, `正在上传: ${file.name}`);
            
            // 1. 上传文件
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
            
            updateProgress(i, 50, `文件上传成功，正在添加到任务...`);
            
            // 2. 添加结果文件到任务
            const fileUrl = data.data.file_url;
            const fileName = data.data.file_name;
            
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
            
            updateProgress(i, 100, `文件 ${i + 1}/${totalFiles} 处理完成`);
            results.push({ fileName, fileUrl });
        }
        
        // 如果选择了状态，则更新任务状态
        if (status) {
            statusText.textContent = `正在更新任务状态为: ${getStatusText(status)}`;
            totalProgress = 95;
            progressBar.style.width = `${totalProgress}%`;
            progressBar.textContent = `${totalProgress}%`;
            progressBar.setAttribute('aria-valuenow', totalProgress);
            
            const statusResponse = await fetch(`${API_BASE_URL}/clips/${id}/status`, {
                method: 'PUT',
                headers: {
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify({ status })
            });
            
            if (!statusResponse.ok) {
                console.warn(`更新任务状态失败，状态码: ${statusResponse.status}`);
                statusText.textContent = `警告: 更新任务状态失败，但文件已上传成功`;
            } else {
                const statusData = await statusResponse.json();
                if (!statusData.success) {
                    console.warn(`更新任务状态失败: ${statusData.error || '未知错误'}`);
                    statusText.textContent = `警告: 更新任务状态失败，但文件已上传成功`;
                } else {
                    statusText.textContent = `任务状态更新成功`;
                }
            }
        }
        
        // 完成所有操作
        totalProgress = 100;
        progressBar.style.width = `${totalProgress}%`;
        progressBar.textContent = `${totalProgress}%`;
        progressBar.setAttribute('aria-valuenow', totalProgress);
        progressBar.classList.remove('progress-bar-animated');
        statusText.textContent = `上传完成: 成功处理 ${results.length} 个文件`;
        detailText.textContent = '';
        
        // 短暂延迟后关闭模态框并刷新
        setTimeout(() => {
            showSuccess(`成功上传 ${results.length} 个结果文件`);
            uploadResultModal.hide();
            
            // 重置进度条
            progressContainer.classList.add('d-none');
            submitUpload.disabled = false;
            
            // 刷新任务列表并自动打开任务详情
            fetchClips().then(() => viewClipDetail(id));
        }, 1500);
        
    } catch (error) {
        console.error('Error:', error);
        
        // 更新进度条显示错误
        progressBar.classList.remove('progress-bar-animated');
        progressBar.classList.remove('bg-primary');
        progressBar.classList.add('bg-danger');
        statusText.textContent = `上传失败: ${error.message}`;
        
        // 启用提交按钮
        submitUpload.disabled = false;
        
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
