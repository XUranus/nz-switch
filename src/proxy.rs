use anyhow::Result;
use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::info;
use crate::format;

/// 代理配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyConfig {
    /// 代理地址 (如 http://127.0.0.1:7890)
    pub address: String,

    /// 代理类型 (http / socks5)
    #[serde(default = "default_proxy_type")]
    pub proxy_type: String,
}

fn default_proxy_type() -> String {
    "http".to_string()
}

// ─── 子命令实现 ──────────────────────────────────────────────────────

/// 设置代理地址
pub fn set_proxy(address: &str) -> Result<()> {
    let address = address.to_string();
    let proxy_type = if address.starts_with("socks") {
        "socks5".to_string()
    } else {
        "http".to_string()
    };

    crate::config::mutate_current_profile(|profile| {
        profile.proxy = Some(ProxyConfig {
            address: address.clone(),
            proxy_type,
        });
    })?;

    println!("{} 代理地址已设置为: {}", "✅".green(), address.cyan());
    println!("  运行 {} 来应用", "nz-switch switch cn".cyan());

    Ok(())
}

/// 开启代理环境变量
pub fn enable_proxy() -> Result<()> {
    let cfg = crate::config::AppConfig::load()?;
    let profile = crate::profile::resolve_profile(&cfg.current_profile)?;

    match &profile.proxy {
        Some(p) => {
            apply_proxy(&Some(p.clone()))?;
            println!("{} 代理已开启", "✅".green());
        }
        None => {
            println!("{} 当前 profile 未配置代理地址", "⚠".yellow());
            println!("  运行 {} 来设置", "nz-switch proxy set <address>".cyan());
        }
    }

    Ok(())
}

/// 关闭代理环境变量
pub fn disable_proxy() -> Result<()> {
    clear_proxy_env();
    println!("{} 代理已关闭", "✅".green());
    Ok(())
}

/// 测试代理连通性
pub fn test_proxy() -> Result<()> {
    println!("{}", "🔌 代理连通性测试".cyan().bold());
    println!();

    let tests = vec![
        ("GitHub", "https://github.com"),
        ("Google", "https://www.google.com"),
        ("Docker Hub", "https://hub.docker.com"),
        ("PyPI (清华)", "https://pypi.tuna.tsinghua.edu.cn"),
        ("crates.io (ustc)", "https://mirrors.ustc.edu.cn/crates.io-index/"),
        ("npmmirror", "https://registry.npmmirror.com"),
    ];

    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()?;

    let mut table = format::new_table(&["状态", "目标", "延迟", "结果"]);

    for (name, url) in &tests {
        let start = std::time::Instant::now();
        match client.head(*url).send() {
            Ok(resp) => {
                let ms = start.elapsed().as_millis() as u64;
                let status = resp.status();
                table.add_row(vec![
                    format::latency_icon(ms),
                    name.cyan().to_string(),
                    format!("{}ms", ms),
                    format!("HTTP {}", status.as_u16()),
                ]);
            }
            Err(e) => {
                table.add_row(vec![
                    format::err(),
                    name.cyan().to_string(),
                    "-".to_string(),
                    format!("失败: {e}"),
                ]);
            }
        }
    }

    println!("{table}");
    println!();

    let mut env_table = format::new_table(&["变量", "值"]);
    for key in &["HTTP_PROXY", "HTTPS_PROXY", "ALL_PROXY"] {
        match std::env::var(key) {
            Ok(val) => { env_table.add_row(vec![key.cyan().to_string(), val]); }
            Err(_) => { env_table.add_row(vec![key.dimmed().to_string(), "(未设置)".dimmed().to_string()]); }
        }
    }
    println!("{env_table}");

    Ok(())
}

// ─── 内部函数 ────────────────────────────────────────────────────────

/// 应用代理配置（设置环境变量并持久化到 shell 配置）
pub fn apply_proxy(proxy: &Option<ProxyConfig>) -> Result<()> {
    match proxy {
        Some(p) => {
            println!("  {}", "代理:".bold());

            let env_vars = if p.proxy_type.starts_with("socks") {
                // socks4, socks5, socks5h 等所有 SOCKS 变体使用 ALL_PROXY
                vec![("ALL_PROXY", p.address.as_str())]
            } else {
                vec![
                    ("HTTP_PROXY", p.address.as_str()),
                    ("HTTPS_PROXY", p.address.as_str()),
                ]
            };

            for (key, value) in &env_vars {
                // SAFETY: 单线程上下文调用
                unsafe { std::env::set_var(key, *value); }
                info!("set proxy env {}={}", key, value);
                println!("    {} {} = {}", "✓".green(), key.cyan(), value);
                // 持久化到 shell 配置
                if let Err(e) = crate::env::write_single_to_shell(key, value) {
                    info!("failed to persist proxy env {} to shell config: {}", key, e);
                }
            }

            let no_proxy = "localhost,127.0.0.1,10.0.0.0/8,172.16.0.0/12,192.168.0.0/16";
            // SAFETY: 单线程上下文调用
            unsafe { std::env::set_var("NO_PROXY", no_proxy); }
            println!("    {} NO_PROXY = {}", "✓".green(), no_proxy);
            if let Err(e) = crate::env::write_single_to_shell("NO_PROXY", no_proxy) {
                info!("failed to persist NO_PROXY to shell config: {}", e);
            }
        }
        None => {
            println!("  {} 代理: 未配置", "ℹ".blue());
            clear_proxy_env();
        }
    }

    Ok(())
}

/// 清除代理环境变量（同时从 shell 配置中移除）
fn clear_proxy_env() {
    for key in &["HTTP_PROXY", "HTTPS_PROXY", "ALL_PROXY", "NO_PROXY",
                 "http_proxy", "https_proxy", "all_proxy", "no_proxy"] {
        // SAFETY: 单线程上下文调用
        unsafe { std::env::remove_var(key); }
        // 从 shell 配置中移除（忽略错误）
        let _ = crate::env::remove_from_shell_config(key);
    }
}
