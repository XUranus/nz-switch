// 工具配置文件路径集中管理

use anyhow::Result;
use std::path::PathBuf;

// ─── 各工具配置文件路径 ──────────────────────────────────────────────

/// pip 配置文件路径（跨平台）
/// - macOS: ~/Library/Application Support/pip/pip.conf
/// - Windows: %APPDATA%\pip\pip.ini
/// - Linux: ~/.config/pip/pip.conf
pub fn pip_config_path() -> Result<PathBuf> {
    let home = crate::home_dir()?;
    let os = std::env::consts::OS;
    match os {
        "macos" => Ok(home
            .join("Library")
            .join("Application Support")
            .join("pip")
            .join("pip.conf")),
        "windows" => {
            let appdata = std::env::var("APPDATA").unwrap_or_else(|_| {
                home.join("AppData")
                    .join("Roaming")
                    .to_string_lossy()
                    .to_string()
            });
            Ok(std::path::PathBuf::from(appdata)
                .join("pip")
                .join("pip.ini"))
        }
        _ => Ok(home.join(".config").join("pip").join("pip.conf")),
    }
}

/// cargo 配置文件路径: ~/.cargo/config.toml
pub fn cargo_config_path() -> Result<PathBuf> {
    Ok(crate::home_dir()?.join(".cargo").join("config.toml"))
}

/// npm/pnpm 配置文件路径: ~/.npmrc
pub fn npmrc_path() -> Result<PathBuf> {
    Ok(crate::home_dir()?.join(".npmrc"))
}

/// yarn 配置文件路径: ~/.yarnrc
pub fn yarnrc_path() -> Result<PathBuf> {
    Ok(crate::home_dir()?.join(".yarnrc"))
}

/// conda 配置文件路径: ~/.condarc
pub fn condarc_path() -> Result<PathBuf> {
    Ok(crate::home_dir()?.join(".condarc"))
}

/// Docker 用户级配置路径: ~/.docker/daemon.json
pub fn docker_user_daemon_path() -> Result<PathBuf> {
    Ok(crate::home_dir()?.join(".docker").join("daemon.json"))
}

/// Docker 系统级配置路径: /etc/docker/daemon.json
pub fn docker_sys_daemon_path() -> PathBuf {
    PathBuf::from("/etc/docker/daemon.json")
}

/// Maven settings.xml 路径: ~/.m2/settings.xml
pub fn maven_settings_path() -> Result<PathBuf> {
    Ok(crate::home_dir()?.join(".m2").join("settings.xml"))
}

/// Gradle init.gradle 路径: ~/.gradle/init.gradle
pub fn gradle_init_path() -> Result<PathBuf> {
    Ok(crate::home_dir()?.join(".gradle").join("init.gradle"))
}

/// NuGet 配置文件路径（跨平台）
pub fn nuget_config_path() -> Result<PathBuf> {
    if cfg!(target_os = "windows") {
        let appdata =
            std::env::var("APPDATA").map_err(|_| anyhow::anyhow!("无法获取 APPDATA 环境变量"))?;
        Ok(std::path::PathBuf::from(appdata)
            .join("NuGet")
            .join("NuGet.Config"))
    } else {
        Ok(crate::home_dir()?
            .join(".nuget")
            .join("NuGet")
            .join("NuGet.Config"))
    }
}

/// VS Code settings.json 路径（跨平台）
pub fn vscode_settings_path() -> Result<PathBuf> {
    let home = crate::home_dir()?;
    if cfg!(target_os = "macos") {
        Ok(home
            .join("Library")
            .join("Application Support")
            .join("Code")
            .join("User")
            .join("settings.json"))
    } else if cfg!(target_os = "windows") {
        let appdata = std::env::var("APPDATA").unwrap_or_else(|_| {
            home.join("AppData")
                .join("Roaming")
                .to_string_lossy()
                .to_string()
        });
        Ok(std::path::PathBuf::from(appdata)
            .join("Code")
            .join("User")
            .join("settings.json"))
    } else {
        Ok(home
            .join(".config")
            .join("Code")
            .join("User")
            .join("settings.json"))
    }
}
