use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::profile::Profile;

/// 项目级配置 (.nz-switch.toml)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LocalConfig {
    /// 要使用的全局 profile 名称 (可选，不设置则使用当前全局 profile)
    #[serde(default)]
    pub base_profile: Option<String>,

    /// 覆盖的环境变量
    #[serde(default)]
    pub env: HashMap<String, String>,

    /// 覆盖的镜像源
    #[serde(default)]
    pub mirrors: HashMap<String, String>,

    /// 是否禁用代理 (覆盖全局)
    #[serde(default)]
    pub no_proxy: bool,
}

/// 项目级配置文件名
const LOCAL_CONFIG_FILENAME: &str = ".nz-switch.toml";

/// 在当前目录及父目录中查找 .nz-switch.toml
pub fn find_local_config() -> Option<PathBuf> {
    let start = std::env::current_dir().ok()?;
    find_config_in_or_above(&start)
}

fn find_config_in_or_above(dir: &Path) -> Option<PathBuf> {
    let mut current = dir.to_path_buf();
    for _ in 0..32 { // 最多向上查找 32 层
        let config_path = current.join(LOCAL_CONFIG_FILENAME);
        if config_path.exists() {
            return Some(config_path);
        }
        if !current.pop() {
            break;
        }
    }
    None
}

/// 加载项目级配置
pub fn load_local_config() -> Result<Option<LocalConfig>> {
    let path = match find_local_config() {
        Some(p) => p,
        None => return Ok(None),
    };

    let content = std::fs::read_to_string(&path)
        .with_context(|| format!("读取项目配置失败: {}", path.display()))?;

    let cfg: LocalConfig = toml::from_str(&content)
        .with_context(|| format!("解析项目配置失败: {}", path.display()))?;

    Ok(Some(cfg))
}

/// 创建项目级配置文件
pub fn create_local_config() -> Result<PathBuf> {
    let path = PathBuf::from(LOCAL_CONFIG_FILENAME);
    if path.exists() {
        anyhow::bail!("项目配置文件已存在: {}", path.display());
    }

    let cfg = LocalConfig::default();
    let content = toml::to_string_pretty(&cfg)?;
    std::fs::write(&path, content)?;

    Ok(path)
}

/// 合并全局 profile 和项目级配置
///
/// 如果 local.base_profile 指定了基础 profile 名称，则以该 profile 为基础进行合并。
/// 否则使用传入的 profile。
pub fn merge_with_local(
    profile: &Profile,
    local: &LocalConfig,
    cfg: &crate::config::AppConfig,
) -> Result<Profile> {
    // 如果指定了 base_profile，从配置中查找
    let mut merged = if let Some(base_name) = &local.base_profile {
        cfg.profiles
            .get(base_name)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!(
                "项目配置指定的 base_profile '{}' 不存在，可用: {}",
                base_name,
                cfg.profiles.keys().cloned().collect::<Vec<_>>().join(", ")
            ))?
    } else {
        profile.clone()
    };

    // 合并环境变量 (项目级覆盖全局)
    for (key, value) in &local.env {
        merged.env.insert(key.clone(), value.clone());
    }

    // 合并镜像源 (项目级覆盖全局)
    for (key, value) in &local.mirrors {
        merged.mirrors.insert(key.clone(), value.clone());
    }

    // 如果项目级禁用代理
    if local.no_proxy {
        merged.proxy = None;
    }

    Ok(merged)
}

/// 显示项目级配置
pub fn show_local_config() -> Result<()> {
    match find_local_config() {
        Some(path) => {
            println!("📁 项目配置文件: {}", path.display());
            let content = std::fs::read_to_string(&path)?;
            println!();
            println!("{content}");
        }
        None => {
            println!("(未找到项目配置文件 .nz-switch.toml)");
            println!();
            println!("运行 {} 创建项目配置", "nz-switch init --local".cyan());
        }
    }

    Ok(())
}

use colored::Colorize;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_local_config_default_is_empty() {
        let cfg = LocalConfig::default();
        assert!(cfg.env.is_empty());
        assert!(cfg.mirrors.is_empty());
        assert!(cfg.base_profile.is_none());
        assert!(!cfg.no_proxy);
    }

    #[test]
    fn test_merge_with_local_overrides_env() {
        let mut profile = Profile {
            display_name: "test".to_string(),
            env: HashMap::new(),
            mirrors: HashMap::new(),
            proxy: None,
            git: None,
            dns: None,
        };
        profile.env.insert("KEY1".to_string(), "value1".to_string());

        let mut local = LocalConfig::default();
        local.env.insert("KEY1".to_string(), "overridden".to_string());
        local.env.insert("KEY2".to_string(), "new".to_string());

        let cfg = crate::config::AppConfig::default();
        let merged = merge_with_local(&profile, &local, &cfg).unwrap();
        assert_eq!(merged.env.get("KEY1").unwrap(), "overridden");
        assert_eq!(merged.env.get("KEY2").unwrap(), "new");
    }

    #[test]
    fn test_merge_with_local_overrides_mirrors() {
        let mut profile = Profile {
            display_name: "test".to_string(),
            env: HashMap::new(),
            mirrors: HashMap::new(),
            proxy: None,
            git: None,
            dns: None,
        };
        profile.mirrors.insert("pip".to_string(), "old".to_string());

        let mut local = LocalConfig::default();
        local.mirrors.insert("pip".to_string(), "new".to_string());

        let cfg = crate::config::AppConfig::default();
        let merged = merge_with_local(&profile, &local, &cfg).unwrap();
        assert_eq!(merged.mirrors.get("pip").unwrap(), "new");
    }

    #[test]
    fn test_merge_no_proxy_flag() {
        let profile = Profile {
            display_name: "test".to_string(),
            env: HashMap::new(),
            mirrors: HashMap::new(),
            proxy: Some(crate::proxy::ProxyConfig {
                address: "http://127.0.0.1:7890".to_string(),
                proxy_type: "http".to_string(),
            }),
            git: None,
            dns: None,
        };

        let mut local = LocalConfig::default();
        local.no_proxy = true;

        let cfg = crate::config::AppConfig::default();
        let merged = merge_with_local(&profile, &local, &cfg).unwrap();
        assert!(merged.proxy.is_none());
    }

    #[test]
    fn test_merge_with_base_profile() {
        let profile = Profile {
            display_name: "test".to_string(),
            env: HashMap::new(),
            mirrors: HashMap::new(),
            proxy: None,
            git: None,
            dns: None,
        };

        let mut local = LocalConfig::default();
        local.base_profile = Some("cn".to_string());
        local.env.insert("MY_VAR".to_string(), "test".to_string());

        let cfg = crate::config::AppConfig::default();
        let merged = merge_with_local(&profile, &local, &cfg).unwrap();
        // base_profile 指定 cn，cn profile 有 pip/npm 等镜像
        assert!(merged.mirrors.contains_key("pip"));
        assert!(merged.mirrors.contains_key("npm"));
        // local env 仍然被合并
        assert_eq!(merged.env.get("MY_VAR").unwrap(), "test");
    }

    #[test]
    fn test_merge_with_invalid_base_profile() {
        let profile = Profile {
            display_name: "test".to_string(),
            env: HashMap::new(),
            mirrors: HashMap::new(),
            proxy: None,
            git: None,
            dns: None,
        };

        let mut local = LocalConfig::default();
        local.base_profile = Some("nonexistent".to_string());

        let cfg = crate::config::AppConfig::default();
        let result = merge_with_local(&profile, &local, &cfg);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("nonexistent"));
    }

    #[test]
    fn test_local_config_serialization() {
        let cfg = LocalConfig::default();
        let toml_str = toml::to_string_pretty(&cfg).unwrap();
        let parsed: LocalConfig = toml::from_str(&toml_str).unwrap();
        assert!(parsed.env.is_empty());
        assert!(!parsed.no_proxy);
    }
}
