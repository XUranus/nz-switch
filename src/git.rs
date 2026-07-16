use anyhow::Result;
use colored::Colorize;
use serde::{Deserialize, Serialize};
use tracing::info;

/// Git 相关配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitConfig {
    /// GitHub 镜像代理 URL (如 https://ghfast.top/)
    #[serde(default)]
    pub github_mirror: Option<String>,

    /// Git 全局代理
    #[serde(default)]
    pub proxy: Option<String>,
}

/// 应用 Git 配置
pub fn apply_git_config(git: &Option<GitConfig>) -> Result<()> {
    match git {
        Some(g) => {
            println!("  {}", "Git:".bold());

            // 先清除旧的镜像条目，避免残留多个 url.insteadOf
            clear_github_mirror()?;

            // 设置 GitHub 镜像 (git url rewrite)
            if let Some(mirror) = &g.github_mirror {
                let instead_of = format!("{mirror}https://github.com/");
                run_git_config("url.insteadOf", &instead_of, "https://github.com/")?;
                println!("    {} GitHub 镜像: {}", "✓".green(), mirror.cyan());
            }

            // 设置 Git 代理
            if let Some(proxy) = &g.proxy {
                run_git_config("http.proxy", proxy, "")?;
                run_git_config("https.proxy", proxy, "")?;
                println!("    {} Git 代理: {}", "✓".green(), proxy.cyan());
            }
        }
        None => {
            println!("  {} Git: 无自定义配置", "ℹ".blue());
            // 清除 git 代理
            clear_git_proxy()?;
        }
    }

    Ok(())
}

/// 运行 git config 命令
fn run_git_config(key: &str, value: &str, instead_of: &str) -> Result<()> {
    let mut cmd = std::process::Command::new("git");
    cmd.arg("config").arg("--global");

    if !instead_of.is_empty() {
        // url.insteadOf 需要特殊处理
        cmd.arg(format!("url.{value}.insteadOf"));
        cmd.arg(instead_of);
    } else {
        cmd.arg(key);
        cmd.arg(value);
    }

    let output = cmd
        .output()
        .map_err(|e| anyhow::anyhow!("git config 执行失败 (git 是否已安装?): {e}"))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("git config 失败: {}", stderr.trim());
    }

    Ok(())
}

/// 清除 Git 代理配置（仅清除 nz-switch 管理的条目）
fn clear_git_proxy() -> Result<()> {
    for key in &["http.proxy", "https.proxy"] {
        let output = std::process::Command::new("git")
            .arg("config")
            .arg("--global")
            .arg("--unset")
            .arg(key)
            .output()?;

        // 忽略错误（key 不存在时会报错）
        if output.status.success() {
            info!("unset git config {}", key);
        }
    }

    clear_github_mirror()
}

/// 清除 nz-switch 管理的 GitHub 镜像 url.insteadOf 条目
fn clear_github_mirror() -> Result<()> {
    // 匹配已知镜像域名（包含已失效的，用于清理旧配置）
    let known_mirrors = [
        "https://ghfast.top/",
        "https://gh-proxy.com/",
        "https://gh.ddlc.top/",
        "https://mirrors.tuna.tsinghua.edu.cn/",
        "https://mirrors.ustc.edu.cn/",
        "https://ghproxy.com/",        // 已失效，保留用于清理旧配置
        "https://mirror.ghproxy.com/", // 已失效，保留用于清理旧配置
        "https://ghproxy.net/",        // 已失效，保留用于清理旧配置
        "https://hub.fastgit.xyz/",    // 已失效，保留用于清理旧配置
    ];

    let output = std::process::Command::new("git")
        .arg("config")
        .arg("--global")
        .arg("--get-regexp")
        .arg("url\\..*\\.insteadof")
        .output()?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            // line 格式: "url.<mirror>https://github.com/.insteadof https://github.com/"
            // 第一个字段是 key（包含镜像域名），第二个字段是 insteadOf 目标
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 2 {
                continue;
            }
            let url_key = parts[0]; // e.g. "url.https://ghproxy.com/https://github.com/.insteadof"
            let instead_of_target = parts[1]; // e.g. "https://github.com/"

            // 检查 key 中是否包含已知镜像域名
            if known_mirrors.iter().any(|m| url_key.contains(m)) {
                let _ = std::process::Command::new("git")
                    .arg("config")
                    .arg("--global")
                    .arg("--unset")
                    .arg(url_key)
                    .arg(instead_of_target)
                    .output();
                info!("removed nz-switch git url.insteadOf: {instead_of_target}");
            }
        }
    }

    Ok(())
}
