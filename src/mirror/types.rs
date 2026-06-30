// 镜像源数据结构

use serde::Serialize;

/// 镜像源应用结果
#[derive(Debug, Clone)]
pub enum ApplyResult {
    /// 已自动应用（写入配置文件或环境变量）
    Applied,
    /// 需要用户手动执行（附带操作说明）
    ManualRequired(String),
}

/// 单个镜像的延迟结果
#[derive(Debug, Clone, Serialize)]
pub struct MirrorLatency {
    pub name: String,
    pub url: String,
    pub latency_ms: Option<u64>, // None = 超时
}

/// 单个工具的测速结果
#[derive(Debug, Clone, Serialize)]
pub struct MirrorTestResult {
    pub tool: String,
    pub results: Vec<MirrorLatency>,
    pub recommended: Option<String>, // 最快的预设名
}

/// 单个镜像测速结果（用于流式回调）
#[derive(Debug, Clone, Serialize)]
pub struct MirrorSingleResult {
    pub tool: String,
    pub name: String,
    pub url: String,
    pub latency_ms: Option<u64>,
}
