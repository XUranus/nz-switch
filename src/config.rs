use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::profile::Profile;

/// 应用配置（持久化到 ~/.config/nz-switch/config.toml）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// 当前激活的 profile 名称
    pub current_profile: String,

    /// 自定义 profiles
    #[serde(default)]
    pub profiles: HashMap<String, Profile>,
}

impl Default for AppConfig {
    fn default() -> Self {
        let mut profiles = HashMap::new();

        // 内置 cn profile
        profiles.insert("cn".to_string(), cn_profile());
        // 内置 global profile
        profiles.insert("global".to_string(), global_profile());

        Self {
            current_profile: "global".to_string(),
            profiles,
        }
    }
}

impl AppConfig {
    /// 从配置文件加载
    pub fn load() -> Result<Self> {
        let path = config_path()?;
        if !path.exists() {
            // 配置文件不存在，返回默认配置
            return Ok(Self::default());
        }

        let content = std::fs::read_to_string(&path)
            .with_context(|| format!("读取配置文件失败: {}", path.display()))?;

        let cfg: AppConfig = match toml::from_str(&content) {
            Ok(cfg) => cfg,
            Err(e) => {
                eprintln!(
                    "Warning: failed to parse config file ({}): {}. Using default config.",
                    path.display(),
                    e
                );
                Self::default()
            }
        };

        Ok(cfg)
    }

    /// 保存到配置文件
    pub fn save(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("创建配置目录失败: {}", parent.display()))?;
        }

        let content = toml::to_string_pretty(self).context("序列化配置失败")?;

        std::fs::write(path, content)
            .with_context(|| format!("写入配置文件失败: {}", path.display()))?;

        Ok(())
    }
}

/// 获取配置文件路径 (~/.config/nz-switch/config.toml)
pub fn config_path() -> Result<PathBuf> {
    let config_dir = dirs::config_dir().context("无法获取用户配置目录")?;
    Ok(config_dir.join("nz-switch").join("config.toml"))
}

/// 对当前 profile 执行修改并自动保存配置
/// 消除了各模块重复的 load→get_mut→save 模式
pub fn mutate_current_profile<F>(f: F) -> Result<()>
where
    F: FnOnce(&mut Profile),
{
    let config_path = config_path()?;
    let mut cfg = AppConfig::load()?;
    let profile_name = cfg.current_profile.clone();
    let profile = cfg
        .profiles
        .get_mut(&profile_name)
        .ok_or_else(|| anyhow::anyhow!("当前 profile '{profile_name}' 不存在"))?;
    f(profile);
    cfg.save(&config_path)?;
    Ok(())
}

/// 内置 CN profile
fn cn_profile() -> Profile {
    // 环境变量型镜像源（通过 env 类型自动设置）
    // go、rustup、brew、pub、huggingface 等工具通过 mirrors 字段自动设置对应环境变量

    let mut mirrors = HashMap::new();
    // 包管理器 - 文件型
    mirrors.insert("pip".into(), "tuna".into());
    mirrors.insert("npm".into(), "npmmirror".into());
    mirrors.insert("yarn".into(), "npmmirror".into());
    mirrors.insert("pnpm".into(), "npmmirror".into());
    mirrors.insert("cargo".into(), "rsproxy".into());
    mirrors.insert("conda".into(), "tuna".into());
    mirrors.insert("maven".into(), "aliyun".into());
    // 包管理器 - 环境变量型（自动设置对应 env）
    mirrors.insert("go".into(), "goproxy.cn".into());
    mirrors.insert("rustup".into(), "ustc".into());
    mirrors.insert("bun".into(), "npmmirror".into());
    mirrors.insert("pub".into(), "tuna".into());
    mirrors.insert("huggingface".into(), "hf-mirror".into());
    // 容器镜像
    mirrors.insert("docker".into(), "aliyun-hangzhou".into());
    // 平台工具
    mirrors.insert("nodejs".into(), "npmmirror".into());
    mirrors.insert("python".into(), "tuna".into());
    mirrors.insert("vscode".into(), "tuna-open-vsx".into());

    Profile {
        display_name: "中国内地".to_string(),
        env: HashMap::new(), // env 由 mirror 系统自动设置
        mirrors,
        proxy: None,
        git: Some(crate::git::GitConfig {
            github_mirror: Some("https://ghfast.top/".to_string()),
            proxy: None,
        }),
        dns: Some(crate::dns::DnsConfig {
            servers: vec!["223.5.5.5".to_string(), "114.114.114.114".to_string()],
        }),
    }
}

/// 内置 global profile（恢复默认，无任何自定义配置）
fn global_profile() -> Profile {
    Profile {
        display_name: "海外".to_string(),
        env: HashMap::new(),
        mirrors: HashMap::new(),
        proxy: None,
        git: None,
        dns: Some(crate::dns::DnsConfig {
            servers: vec!["8.8.8.8".to_string(), "1.1.1.1".to_string()],
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_has_cn_and_global() {
        let cfg = AppConfig::default();
        assert!(cfg.profiles.contains_key("cn"));
        assert!(cfg.profiles.contains_key("global"));
        assert_eq!(cfg.current_profile, "global");
    }

    #[test]
    fn test_cn_profile_has_mirrors() {
        let cfg = AppConfig::default();
        let cn = cfg.profiles.get("cn").unwrap();
        assert!(cn.mirrors.contains_key("pip"));
        assert!(cn.mirrors.contains_key("npm"));
        assert!(cn.mirrors.contains_key("cargo")); // rsproxy sparse index，安全启用
        assert!(cn.mirrors.contains_key("go"));
        assert!(cn.mirrors.contains_key("rustup"));
    }

    #[test]
    fn test_cn_profile_no_env() {
        // 环境变量由 mirror 系统自动设置，profile 中不再手动定义
        let cfg = AppConfig::default();
        let cn = cfg.profiles.get("cn").unwrap();
        assert!(cn.env.is_empty());
    }

    #[test]
    fn test_cn_profile_no_proxy() {
        // 默认不配置代理，用户可通过 local config 自定义
        let cfg = AppConfig::default();
        let cn = cfg.profiles.get("cn").unwrap();
        assert!(cn.proxy.is_none());
    }

    #[test]
    fn test_cn_profile_has_github_mirror() {
        let cfg = AppConfig::default();
        let cn = cfg.profiles.get("cn").unwrap();
        let git = cn.git.as_ref().expect("cn profile should have git config");
        assert!(git.github_mirror.is_some());
        let mirror = git.github_mirror.as_ref().unwrap();
        assert!(
            mirror.starts_with("https://"),
            "github_mirror should use HTTPS"
        );
        assert!(mirror.ends_with('/'), "github_mirror should end with /");
    }

    #[test]
    fn test_global_profile_is_clean() {
        let cfg = AppConfig::default();
        let global = cfg.profiles.get("global").unwrap();
        assert!(global.env.is_empty());
        assert!(global.mirrors.is_empty());
        assert!(global.proxy.is_none());
        assert!(global.git.is_none());
    }

    #[test]
    fn test_config_serialization_roundtrip() {
        let cfg = AppConfig::default();
        let toml_str = toml::to_string_pretty(&cfg).unwrap();
        let parsed: AppConfig = toml::from_str(&toml_str).unwrap();
        assert_eq!(parsed.current_profile, cfg.current_profile);
        assert_eq!(parsed.profiles.len(), cfg.profiles.len());
    }

    #[test]
    fn test_config_path_is_valid() {
        let path = config_path().unwrap();
        assert!(path.to_string_lossy().contains("nz-switch"));
        assert!(path.to_string_lossy().ends_with("config.toml"));
    }

    #[test]
    fn test_save_and_load_config() {
        let dir = std::env::temp_dir().join("nz-switch-test");
        let _ = std::fs::remove_dir_all(&dir);

        let cfg = AppConfig::default();
        let path = dir.join("config.toml");
        cfg.save(&path).unwrap();

        let content = std::fs::read_to_string(&path).unwrap();
        let loaded: AppConfig = toml::from_str(&content).unwrap();
        assert_eq!(loaded.current_profile, "global");
        assert!(loaded.profiles.contains_key("cn"));

        let _ = std::fs::remove_dir_all(&dir);
    }
}
