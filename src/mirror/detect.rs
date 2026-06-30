// 镜像源实时检测

use super::env_vars;
use super::paths;
use super::config;
use super::parse::{parse_ini_value, parse_key_eq_value, parse_maven_mirror_url, parse_apt_sources};

/// 读取系统实际配置，检测当前工具正在使用的镜像源
/// 返回镜像预设名（如能匹配），否则返回 None
pub fn detect_current_mirror(tool: &str) -> Option<String> {
    let mirror_def = config::find_mirror_def(tool)?;
    let config_type = mirror_def.config_type.as_str();

    let raw_url = match config_type {
        "file" => detect_file_mirror(tool),
        "env" => detect_env_mirror(tool),
        "manual" => detect_manual_mirror(tool),
        _ => None,
    };

    // 没有检测到任何配置 → 回退到官方默认源
    let raw_url = match raw_url {
        Some(url) => url,
        None => {
            // 找 "官方" 条目，返回其 name
            return mirror_def.mirrors.iter()
                .find(|m| m.enabled && m.name == "official")
                .map(|m| m.name.clone());
        }
    };

    // 先检查 raw_url 本身是否就是一个预设名（如 pip.conf 中 index-url = tsinghua）
    if mirror_def.mirrors.iter().any(|m| m.name == raw_url && m.enabled) {
        return Some(raw_url);
    }

    // 尝试反向匹配为预设名
    if let matched @ Some(_) = reverse_match_mirror(mirror_def, &raw_url) {
        return matched;
    }

    // 匹配失败：检查是否为官方源 URL（normalized 或前缀比较）
    let normalized = raw_url.trim_end_matches('/');
    mirror_def.mirrors.iter()
        .find(|m| m.enabled && m.name == "official")
        .filter(|m| {
            let entry_norm = m.url.trim_end_matches('/');
            entry_norm == normalized
                || normalized.starts_with(entry_norm)
                || entry_norm.starts_with(normalized)
        })
        .map(|m| m.name.clone())
}

/// 从文件型配置中读取当前镜像 URL
fn detect_file_mirror(tool: &str) -> Option<String> {
    match tool {
        "pip" => {
            let path = paths::pip_config_path().ok()?;
            let content = std::fs::read_to_string(&path).ok()?;
            parse_ini_value(&content, "global", "index-url")
        }
        "npm" | "pnpm" => {
            let path = paths::npmrc_path().ok()?;
            let content = std::fs::read_to_string(&path).ok()?;
            parse_key_eq_value(&content, "registry")
        }
        "yarn" => {
            let path = paths::yarnrc_path().ok()?;
            let content = std::fs::read_to_string(&path).ok()?;
            // yarn 格式: registry "https://..."
            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with("registry ") {
                    let url = trimmed.trim_start_matches("registry ").trim();
                    let url = url.trim_matches('"');
                    if !url.is_empty() {
                        return Some(url.to_string());
                    }
                }
            }
            None
        }
        "cargo" => {
            let path = paths::cargo_config_path().ok()?;
            let content = std::fs::read_to_string(&path).ok()?;
            // registry = "sparse+https://..." 或 "git+https://..."
            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with("registry") && trimmed.contains('=') {
                    let val = trimmed.split_once('=')?.1.trim();
                    let val = val.trim_matches('"');
                    // 去掉协议前缀
                    let url = val
                        .strip_prefix("sparse+").or_else(|| val.strip_prefix("git+"))
                        .unwrap_or(val);
                    if url.starts_with("http") {
                        return Some(url.trim_end_matches('/').to_string());
                    }
                }
            }
            None
        }
        "conda" => {
            let path = paths::condarc_path().ok()?;
            let content = std::fs::read_to_string(&path).ok()?;
            // 从 default_channels 或 custom_channels 提取 base URL
            for line in content.lines() {
                let trimmed = line.trim().trim_start_matches("- ");
                if !trimmed.starts_with("http") { continue; }
                let base = if let Some(idx) = trimmed.find("/pkgs") {
                    &trimmed[..idx]
                } else if let Some(idx) = trimmed.find("/cloud") {
                    &trimmed[..idx]
                } else {
                    trimmed.trim_end_matches('/')
                };
                if !base.is_empty() {
                    return Some(base.to_string());
                }
            }
            None
        }
        "maven" => {
            let path = paths::maven_settings_path().ok()?;
            let content = std::fs::read_to_string(&path).ok()?;
            parse_maven_mirror_url(&content)
        }
        "gradle" => {
            let path = paths::gradle_init_path().ok()?;
            let content = std::fs::read_to_string(&path).ok()?;
            // maven { url '...' }
            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed.contains("maven") && trimmed.contains("url") {
                    if let Some(start) = trimmed.find("'").or_else(|| trimmed.find('"')) {
                        let quote_char = trimmed.as_bytes()[start] as char;
                        let rest = &trimmed[start + 1..];
                        if let Some(end) = rest.find(quote_char) {
                            return Some(rest[..end].to_string());
                        }
                    }
                }
            }
            None
        }
        "docker" => {
            // 优先检查用户级配置 (~/.docker/daemon.json)
            let home_path = paths::docker_user_daemon_path().ok();
            let sys_path = paths::docker_sys_daemon_path();

            let path = if home_path.as_ref().is_some_and(|p| p.exists()) {
                home_path.unwrap()
            } else if sys_path.exists() {
                sys_path
            } else {
                return None;
            };

            let content = std::fs::read_to_string(&path).ok()?;
            let config: serde_json::Value = serde_json::from_str(&content).ok()?;
            let mirrors = config.get("registry-mirrors")?.as_array()?;
            mirrors.first()?.as_str().map(|s| s.to_string())
        }
        _ => None,
    }
}

/// 从环境变量中读取当前镜像 URL
fn detect_env_mirror(tool: &str) -> Option<String> {
    let var_name = env_vars::primary_env_var(tool)?;

    // 优先从进程环境读取
    let val = std::env::var(var_name).ok()
        .filter(|v| !v.is_empty())
        // 回退：从 shell 配置文件读取 export VAR='...'
        .or_else(|| read_env_from_shell_rc(var_name))?;

    // go proxy 格式: "https://goproxy.cn,direct" → 取第一个
    let url = val.split(',').next().unwrap_or(&val);
    Some(url.to_string())
}

/// 从 shell 配置文件（.zshrc/.bashrc）中读取 export VAR='value'
fn read_env_from_shell_rc(var_name: &str) -> Option<String> {
    let home = crate::home_dir().ok()?;
    let rc_files = [
        home.join(".zshrc"),
        home.join(".bashrc"),
        home.join(".bash_profile"),
        home.join(".profile"),
    ];

    let export_sq = format!("export {var_name}='");
    let export_dq = format!("export {var_name}=\"");
    let export_eq = format!("export {var_name}=");

    for rc_path in &rc_files {
        let content = match std::fs::read_to_string(rc_path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        // 从后往前找，取最后一次赋值（最新的覆盖旧的）
        for line in content.lines().rev() {
            let trimmed = line.trim();
            if trimmed.starts_with(&export_sq) {
                let rest = &trimmed[export_sq.len()..];
                if let Some(end) = rest.find('\'') {
                    let val = rest[..end].trim();
                    if !val.is_empty() { return Some(val.to_string()); }
                }
            } else if trimmed.starts_with(&export_dq) {
                let rest = &trimmed[export_dq.len()..];
                if let Some(end) = rest.find('"') {
                    let val = rest[..end].trim();
                    if !val.is_empty() { return Some(val.to_string()); }
                }
            } else if trimmed.starts_with(&export_eq)
                && !trimmed.starts_with(&export_sq)
                && !trimmed.starts_with(&export_dq)
            {
                // export VAR=value（无引号）
                // 截断行内注释：export VAR=value # comment → value
                let raw = &trimmed[export_eq.len()..];
                let val = if let Some(idx) = raw.find(" #") {
                    raw[..idx].trim()
                } else {
                    raw.trim()
                };
                if !val.is_empty() { return Some(val.to_string()); }
            }
        }
    }
    None
}

/// 从 manual 类型工具的系统配置中检测当前镜像 URL
fn detect_manual_mirror(tool: &str) -> Option<String> {
    match tool {
        "apt" => {
            // 传统格式: /etc/apt/sources.list: deb https://mirrors.tuna.tsinghua.edu.cn/ubuntu/ noble main ...
            let urls = parse_apt_sources("/etc/apt/sources.list");
            if let Some(url) = urls.first() {
                return Some(url.clone());
            }
            // deb822 格式: /etc/apt/sources.list.d/*.sources: URIs: https://...
            if let Ok(entries) = std::fs::read_dir("/etc/apt/sources.list.d") {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().map(|e| e == "sources").unwrap_or(false) {
                        if let Ok(content) = std::fs::read_to_string(&path) {
                            for line in content.lines() {
                                let trimmed = line.trim();
                                if let Some(rest) = trimmed.strip_prefix("URIs:") {
                                    let url = rest.trim();
                                    if url.starts_with("http") {
                                        return Some(url.trim_end_matches('/').to_string());
                                    }
                                }
                            }
                        }
                    }
                }
            }
            None
        }
        "choco" => {
            // Windows: %ProgramData%\chocolatey\config\chocolatey.config
            if cfg!(target_os = "windows") {
                let program_data = std::env::var("ProgramData")
                    .unwrap_or_else(|_| r"C:\ProgramData".to_string());
                let path = std::path::PathBuf::from(program_data)
                    .join("chocolatey").join("config").join("chocolatey.config");
                let content = std::fs::read_to_string(&path).ok()?;
                // <source id="mirror" value="https://..." />
                for line in content.lines() {
                    let trimmed = line.trim();
                    if trimmed.contains("source") && trimmed.contains("value=") {
                        if let Some(start) = trimmed.find("value=\"") {
                            let rest = &trimmed[start + 7..];
                            if let Some(end) = rest.find('"') {
                                let url = &rest[..end];
                                if url.starts_with("http") && !url.contains("chocolatey.org") {
                                    return Some(url.to_string());
                                }
                            }
                        }
                    }
                }
            }
            None
        }
        "nuget" => {
            let path = paths::nuget_config_path().ok()?;
            let content = std::fs::read_to_string(&path).ok()?;
            // <add key="Source" value="https://..." />
            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed.contains("key=") && trimmed.contains("value=") && trimmed.contains("http") {
                    if let Some(start) = trimmed.find("value=\"") {
                        let rest = &trimmed[start + 7..];
                        if let Some(end) = rest.find('"') {
                            let url = &rest[..end];
                            if url.starts_with("http") && !url.contains("nuget.org") {
                                return Some(url.to_string());
                            }
                        }
                    }
                }
            }
            None
        }
        "rubygems" => {
            // gem sources 输出: * https://mirrors.tuna.tsinghua.edu.cn/rubygems/
            let output = std::process::Command::new("gem")
                .args(["sources", "--list"])
                .output().ok()?;
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                let trimmed = line.trim().trim_start_matches("* ").trim_start_matches("- ");
                if trimmed.starts_with("http") && !trimmed.contains("rubygems.org") {
                    return Some(trimmed.to_string());
                }
            }
            None
        }
        "composer" => {
            // composer config -g repo.packagist.url
            let output = std::process::Command::new("composer")
                .args(["config", "-g", "repositories.packagist.url"])
                .output().ok()?;
            let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if stdout.starts_with("http") {
                return Some(stdout);
            }
            None
        }
        "cocoapods" => {
            // pod repo list 输出中有 URL
            let output = std::process::Command::new("pod")
                .args(["repo", "list"])
                .output().ok()?;
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with("http") && trimmed.contains("CocoaPods") {
                    return Some(trimmed.to_string());
                }
            }
            None
        }
        "vscode" => {
            let path = paths::vscode_settings_path().ok()?;
            let content = std::fs::read_to_string(&path).ok()?;
            // "extensions.gallery": { "serviceUrl": "https://..." }
            if let Some(start) = content.find("serviceUrl") {
                let rest = &content[start..];
                if let Some(url_start) = rest.find("http") {
                    let url_part = &rest[url_start..];
                    if let Some(end) = url_part.find('"') {
                        return Some(url_part[..end].to_string());
                    }
                }
            }
            None
        }
        "python" => {
            // python manual 类型是指 Python 安装包下载地址，无法自动检测
            None
        }
        "android-maven" | "android-gradle" | "swift" | "k8s-gcr" | "k8s-registry" | "ghcr" | "quay" => {
            // 项目级别配置，无法全局检测
            None
        }
        _ => None,
    }
}

/// 反向匹配：将 URL 匹配回镜像预设名
fn reverse_match_mirror(def: &config::MirrorDef, raw_url: &str) -> Option<String> {
    let normalized = raw_url.trim_end_matches('/');

    for entry in &def.mirrors {
        if !entry.enabled { continue; }
        let entry_normalized = entry.url.trim_end_matches('/');
        // 精确匹配
        if normalized == entry_normalized {
            return Some(entry.name.clone());
        }
        // 带 /simple 后缀匹配 (pip)
        if format!("{entry_normalized}/simple") == normalized {
            return Some(entry.name.clone());
        }
        if normalized == entry_normalized.strip_suffix("/simple").unwrap_or(entry_normalized) {
            return Some(entry.name.clone());
        }
    }

    // 双向前缀匹配：处理检测端去掉了路径后缀（如 conda 的 /pkgs/main）的情况
    for entry in &def.mirrors {
        if !entry.enabled { continue; }
        let entry_normalized = entry.url.trim_end_matches('/');
        if normalized.len() > 8 && (normalized.starts_with(entry_normalized) || entry_normalized.starts_with(normalized)) {
            return Some(entry.name.clone());
        }
    }

    // 域名级别匹配：仅在 URL 无明显路径时尝试
    let raw_host = extract_host(raw_url);
    let raw_path = normalized.strip_prefix("https://").or_else(|| normalized.strip_prefix("http://"))
        .and_then(|s| s.split('/').nth(1)).unwrap_or("");
    if raw_path.is_empty() {
        for entry in &def.mirrors {
            if !entry.enabled { continue; }
            if let (Some(entry_host), Some(raw_host)) = (extract_host(&entry.url), raw_host) {
                if entry_host == raw_host {
                    return Some(entry.name.clone());
                }
            }
        }
    }

    None
}

/// 从 URL 中提取 host 部分
fn extract_host(url: &str) -> Option<&str> {
    let without_proto = url.strip_prefix("https://").or_else(|| url.strip_prefix("http://"))?;
    Some(without_proto.split('/').next().unwrap_or(without_proto))
}

/// 检测所有已安装工具的当前镜像源
pub fn detect_all_mirrors() -> std::collections::HashMap<String, String> {
    let installed = config::installed_tools();
    let mut result = std::collections::HashMap::new();
    for tool in installed {
        if let Some(mirror) = detect_current_mirror(&tool) {
            result.insert(tool, mirror);
        }
    }
    result
}
