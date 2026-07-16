use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::dns::DnsConfig;
use crate::git::GitConfig;
use crate::proxy::ProxyConfig;

/// 一个环境 Profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    /// 显示名称
    pub display_name: String,

    /// 环境变量 (KEY -> VALUE)
    #[serde(default)]
    pub env: HashMap<String, String>,

    /// 镜像源 (工具名 -> 镜像地址/预设名)
    #[serde(default)]
    pub mirrors: HashMap<String, String>,

    /// 代理配置
    #[serde(default)]
    pub proxy: Option<ProxyConfig>,

    /// Git 配置
    #[serde(default)]
    pub git: Option<GitConfig>,

    /// DNS 配置
    #[serde(default)]
    pub dns: Option<DnsConfig>,
}

/// 根据名称解析 profile（先查用户自定义，再查内置）
pub fn resolve_profile(name: &str) -> Result<Profile> {
    let cfg = crate::config::AppConfig::load()?;

    cfg.profiles.get(name).cloned().with_context(|| {
        format!(
            "未找到 profile '{}'. 可用的 profiles: {}",
            name,
            cfg.profiles.keys().cloned().collect::<Vec<_>>().join(", ")
        )
    })
}
