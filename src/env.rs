use anyhow::Result;
use std::collections::HashMap;
use std::path::PathBuf;
use colored::Colorize;
use tracing::info;
use crate::format;

// ─── Shell 辅助函数 ──────────────────────────────────────────────────

/// 检测当前 shell 类型
fn detect_shell() -> Option<String> {
    std::env::var("SHELL").ok().and_then(|s| {
        std::path::Path::new(&s)
            .file_name()
            .map(|f| f.to_string_lossy().to_string())
    })
}

/// 获取 shell 配置文件路径
fn shell_rc_path() -> Option<PathBuf> {
    // Windows: 优先使用 PowerShell profile
    #[cfg(target_os = "windows")]
    {
        // PowerShell 7+ (pwsh): ~/Documents/PowerShell/Microsoft.PowerShell_profile.ps1
        // Windows PowerShell 5.x: ~/Documents/WindowsPowerShell/Microsoft.PowerShell_profile.ps1
        if let Some(docs) = dirs::document_dir() {
            let pwsh_profile = docs.join("PowerShell").join("Microsoft.PowerShell_profile.ps1");
            if pwsh_profile.exists() {
                return Some(pwsh_profile);
            }
            let win_ps_profile = docs.join("WindowsPowerShell").join("Microsoft.PowerShell_profile.ps1");
            if win_ps_profile.exists() {
                return Some(win_ps_profile);
            }
            // 如果都不存在，创建 PowerShell 7+ profile 路径
            return Some(pwsh_profile);
        }
        None
    }

    #[cfg(not(target_os = "windows"))]
    {
        let shell = detect_shell();
        match shell.as_deref() {
            Some("zsh") => dirs::home_dir().map(|h| h.join(".zshrc")),
            Some("bash") => {
                // macOS 使用 .bash_profile，Linux 使用 .bashrc
                let home = dirs::home_dir()?;
                let profile = home.join(".bash_profile");
                if profile.exists() {
                    Some(profile)
                } else {
                    Some(home.join(".bashrc"))
                }
            }
            Some("fish") => dirs::home_dir().map(|h| h.join(".config").join("fish").join("config.fish")),
            _ => {
                // 回退到 .profile
                dirs::home_dir().map(|h| h.join(".profile"))
            }
        }
    }
}

/// 验证环境变量名是否合法 (POSIX: [A-Za-z_][A-Za-z0-9_]*)
fn is_valid_env_key(key: &str) -> bool {
    if key.is_empty() {
        return false;
    }
    let mut chars = key.chars();
    if let Some(first) = chars.next() {
        if !first.is_ascii_alphabetic() && first != '_' {
            return false;
        }
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

// ─── 子命令实现 ──────────────────────────────────────────────────────

/// 显示当前 profile 的环境变量
pub fn show_env() -> Result<()> {
    let cfg = crate::config::AppConfig::load()?;
    let profile = crate::profile::resolve_profile(&cfg.current_profile)?;

    println!("{}", "🔑 当前 profile 环境变量".cyan().bold());
    println!("  Profile: {}", profile.display_name);
    println!();

    if profile.env.is_empty() {
        println!("  (无自定义环境变量)");
    } else {
        let mut table = format::new_table(&["环境变量", "值", "状态"]);
        for (key, value) in &profile.env {
            let active = std::env::var(key).ok();
            let status = match active {
                Some(ref v) if v == value => format::ok(),
                Some(_) => "⚠️ (值不一致)".yellow().to_string(),
                None => "❌ (未生效)".red().to_string(),
            };
            table.add_row(vec![key.cyan().to_string(), value.to_string(), status]);
        }
        println!("{table}");
    }

    Ok(())
}

/// 设置环境变量 (写入当前 profile 配置)
pub fn set_env(key: &str, value: &str) -> Result<()> {
    if !is_valid_env_key(key) {
        anyhow::bail!("无效的环境变量名: {key} (必须匹配 [A-Za-z_][A-Za-z0-9_]*)");
    }

    let mut cfg = crate::config::AppConfig::load()?;
    let profile_name = cfg.current_profile.clone();

    let profile = cfg.profiles.get_mut(&profile_name)
        .ok_or_else(|| anyhow::anyhow!("当前 profile '{profile_name}' 不存在"))?;
    profile.env.insert(key.to_string(), value.to_string());

    // 先克隆 env_map，释放对 cfg 的可变借用
    let env_map = profile.env.clone();

    // 保存 config 文件（确保应用状态一致）
    let config_path = crate::config::config_path()?;
    cfg.save(&config_path)?;

    // 再写入 shell 配置
    write_to_shell_config(&env_map)?;

    // SAFETY: nz-switch CLI 和 Tauri 命令在单线程上下文中调用此函数，
    // 不会与其他线程并发读写环境变量。
    unsafe { std::env::set_var(key, value); }

    println!("{} 环境变量已设置: {} = {}", "✅".green(), key.cyan(), value);

    Ok(())
}

/// 删除环境变量 (从当前 profile 配置中移除)
pub fn unset_env(key: &str) -> Result<()> {
    let mut cfg = crate::config::AppConfig::load()?;
    let profile_name = cfg.current_profile.clone();

    if let Some(profile) = cfg.profiles.get_mut(&profile_name) {
        if profile.env.remove(key).is_some() {
            let config_path = crate::config::config_path()?;
            cfg.save(&config_path)?;

            // SAFETY: 单线程上下文调用
            unsafe { std::env::remove_var(key); }

            remove_from_shell_config(key)?;

            println!("{} 环境变量已删除: {}", "✅".green(), key.cyan());
        } else {
            println!("{} 环境变量 {} 不存在于当前 profile", "⚠".yellow(), key.cyan());
        }
    }

    Ok(())
}

/// 列出当前 shell 中所有 proxy 相关的环境变量
pub fn show_proxy_env() -> Result<()> {
    println!("{}", "🔌 Proxy 相关环境变量".cyan().bold());
    println!();

    let proxy_keys = [
        "HTTP_PROXY", "http_proxy",
        "HTTPS_PROXY", "https_proxy",
        "ALL_PROXY", "all_proxy",
        "NO_PROXY", "no_proxy",
        "SOCKS_PROXY", "socks_proxy",
    ];

    let mut found = false;
    for key in &proxy_keys {
        if let Ok(val) = std::env::var(key) {
            println!("  {} = {}", key.cyan(), val);
            found = true;
        }
    }

    if !found {
        println!("  (未设置任何 proxy 环境变量)");
    }

    Ok(())
}

// ─── 内部函数 ────────────────────────────────────────────────────────

/// 应用环境变量
pub fn apply_env_vars(env_vars: &HashMap<String, String>) -> Result<()> {
    if env_vars.is_empty() {
        return Ok(());
    }

    println!("  {}", "环境变量:".bold());

    for (key, value) in env_vars {
        // SAFETY: 单线程上下文调用
        unsafe { std::env::set_var(key, value); }
        info!("set env {}={}", key, value);
        println!("    {} {} = {}", "✓".green(), key.cyan(), value);
    }

    write_to_shell_config(env_vars)?;

    Ok(())
}

/// 清除 shell 配置文件中的 nz-switch 环境变量块
pub fn clear_shell_env_block() -> Result<()> {
    let rc_path = match shell_rc_path() {
        Some(p) => p,
        None => return Ok(()),
    };

    if !rc_path.exists() {
        return Ok(());
    }

    let content = std::fs::read_to_string(&rc_path)?;
    let marker_start = "# >>> nz-switch env start >>>";
    let marker_end = "# <<< nz-switch env end <<<";

    if let (Some(start_idx), Some(end_idx)) = (content.find(marker_start), content.find(marker_end)) {
        let end_idx = end_idx + marker_end.len();
        let mut result = content[..start_idx].to_string();
        if end_idx < content.len() {
            result.push_str(&content[end_idx..]);
        }
        while result.ends_with("\n\n") {
            result.pop();
        }
        std::fs::write(&rc_path, result)?;
        println!("    {} 已从 shell 配置中移除 nz-switch 环境变量块", "✓".green());
    }

    Ok(())
}

/// 将环境变量写入 shell 配置文件
fn write_to_shell_config(env_vars: &HashMap<String, String>) -> Result<()> {
    let rc_path = match shell_rc_path() {
        Some(p) => p,
        None => {
            println!("    {} 无法检测 shell 类型，跳过写入 shell 配置", "⚠".yellow());
            return Ok(());
        }
    };

    if !rc_path.exists() {
        println!("    {} Shell 配置文件不存在: {}, 跳过写入", "⚠".yellow(), rc_path.display());
        return Ok(());
    }

    let content = std::fs::read_to_string(&rc_path)?;

    let marker_start = "# >>> nz-switch env start >>>";
    let marker_end = "# <<< nz-switch env end <<<";

    let mut block = format!("{marker_start}\n");
    for (key, value) in env_vars {
        // C1 fix: 验证 key 合法性
        if !is_valid_env_key(key) {
            tracing::warn!("skipping invalid env key: {}", key);
            continue;
        }
        // C2 fix: 防止 shell 注入 — 单引号转义 + 剥离换行符
        // 单引号内用 '\'' 转义；换行符在单引号字符串中会终止 export 语句，必须移除
        let sanitized = value.replace('\'', "'\\''").replace('\n', " ").replace('\r', "");
        block.push_str(&format!("export {key}='{sanitized}'\n"));
    }
    block.push_str(&format!("{marker_end}\n"));

    let new_content = if let (Some(start_idx), Some(end_idx)) = (content.find(marker_start), content.find(marker_end)) {
        let end_idx = end_idx + marker_end.len();
        let mut result = content[..start_idx].to_string();
        result.push_str(&block);
        if end_idx < content.len() {
            result.push_str(&content[end_idx..]);
            if result.ends_with("\n\n") {
                result.pop();
            }
        }
        result
    } else {
        let mut result = content;
        if !result.ends_with('\n') {
            result.push('\n');
        }
        result.push('\n');
        result.push_str(&block);
        result
    };

    std::fs::write(&rc_path, new_content)?;
    println!("    {} 已写入 {}", "✓".green(), rc_path.display());

    Ok(())
}

/// 将单个环境变量写入 shell 配置文件（追加 export 行，替换已有值）
/// 注意: 这不在 nz-switch marker block 内，用于 proxy 等独立持久化的变量
pub fn write_single_to_shell(key: &str, value: &str) -> Result<()> {
    let rc_path = match shell_rc_path() {
        Some(p) => p,
        None => return Ok(()),
    };

    if !rc_path.exists() {
        return Ok(());
    }

    if !is_valid_env_key(key) {
        tracing::warn!("skipping invalid env key: {}", key);
        return Ok(());
    }

    let sanitized = value.replace('\'', "'\\''").replace('\n', " ").replace('\r', "");
    let export_line = format!("export {key}='{sanitized}'");

    let content = std::fs::read_to_string(&rc_path)?;
    let export_dquote = format!("export {key}=\"");
    let export_squote_prefix = format!("export {key}='");
    let export_eq_prefix = format!("export {key}=");

    let mut found = false;
    let new_content: String = content.lines()
        .map(|line| {
            let trimmed = line.trim();
            if trimmed.starts_with(&export_dquote)
                || trimmed.starts_with(&export_squote_prefix)
                || (trimmed.starts_with(&export_eq_prefix)
                    && !trimmed.starts_with(&export_dquote)
                    && !trimmed.starts_with(&export_squote_prefix))
            {
                found = true;
                export_line.clone()
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n");

    if found {
        let mut output = new_content;
        if content.ends_with('\n') && !output.ends_with('\n') {
            output.push('\n');
        }
        std::fs::write(&rc_path, output)?;
    } else {
        // 追加
        let mut output = new_content;
        if !output.ends_with('\n') {
            output.push('\n');
        }
        output.push_str(&format!("{export_line}\n"));
        std::fs::write(&rc_path, output)?;
    }

    Ok(())
}

/// 从 shell 配置文件中移除指定环境变量
pub fn remove_from_shell_config(key: &str) -> Result<()> {
    let rc_path = match shell_rc_path() {
        Some(p) => p,
        None => return Ok(()),
    };

    if !rc_path.exists() {
        return Ok(());
    }

    let content = std::fs::read_to_string(&rc_path)?;
    let export_dquote = format!("export {key}=\"");
    let export_squote = format!("export {key}='");
    let export_eq = format!("export {key}=");

    let new_content: String = content.lines()
        .filter(|line| {
            let trimmed = line.trim();
            !trimmed.starts_with(&export_dquote)
                && !trimmed.starts_with(&export_squote)
                && !trimmed.starts_with(&export_eq)
        })
        .collect::<Vec<_>>()
        .join("\n");

    // 保留原始尾部换行
    let mut output = new_content;
    if content.ends_with('\n') && !output.ends_with('\n') {
        output.push('\n');
    }

    std::fs::write(&rc_path, output)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_env_keys() {
        assert!(is_valid_env_key("GOPROXY"));
        assert!(is_valid_env_key("MY_VAR"));
        assert!(is_valid_env_key("_UNDERSCORE"));
        assert!(is_valid_env_key("VAR123"));
    }

    #[test]
    fn test_invalid_env_keys() {
        assert!(!is_valid_env_key(""));
        assert!(!is_valid_env_key("123START"));
        assert!(!is_valid_env_key("HAS-DASH"));
        assert!(!is_valid_env_key("HAS SPACE"));
        assert!(!is_valid_env_key("HAS.DOT"));
    }

    #[test]
    fn test_shell_rc_path_returns_some() {
        // 在 Linux 上应返回 .bashrc 或 .zshrc
        let path = shell_rc_path();
        // 可能是 None（如果 $SHELL 未设置），但不应 panic
        if let Some(p) = path {
            let s = p.to_string_lossy();
            assert!(
                s.contains(".zshrc") || s.contains(".bashrc") || s.contains(".bash_profile")
                    || s.contains("config.fish") || s.contains(".profile"),
                "unexpected rc path: {}",
                s
            );
        }
    }
}
