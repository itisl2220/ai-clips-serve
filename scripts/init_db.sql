-- 创建剪辑任务表
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
);

-- 创建索引
CREATE INDEX IF NOT EXISTS idx_clips_status ON clips(status);
CREATE INDEX IF NOT EXISTS idx_clips_created_at ON clips(created_at);
