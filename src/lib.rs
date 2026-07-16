pub mod cli;
pub mod config;
pub mod dns;
pub mod doctor;
pub mod env;
pub mod format;
pub mod git;
pub mod local_config;
pub mod mirror;
pub mod profile;
pub mod proxy;
// utils.rs removed — unused dead code

use std::path::PathBuf;
use colored::Colorize;

/// 获取 home 目录
pub fn home_dir() -> anyhow::Result<PathBuf> {
    dirs::home_dir().ok_or_else(|| anyhow::anyhow!("无法获取 home 目录"))
}

/// 切换结果
pub struct SwitchResult {
    /// 子系统错误列表
    pub errors: Vec<String>,
    /// 需要手动执行的操作说明
    pub manual_instructions: Vec<String>,
}

/// 切换 profile 的共享逻辑 (CLI 和 Tauri 共用)
/// 切换前先清理上一个 profile 的所有配置，再应用新 profile
/// 始终保存配置，即使部分子系统失败
pub fn switch_profile(profile_name: &str) -> anyhow::Result<SwitchResult> {
    let mut cfg = config::AppConfig::load()?;
    let profile = profile::resolve_profile(profile_name)?;

    // 合并项目级配置（如果存在 .nz-switch.toml）
    let effective_profile = match local_config::load_local_config()? {
        Some(ref lc) => local_config::merge_with_local(&profile, lc, &cfg)?,
        None => profile,
    };

    let mut errors: Vec<String> = Vec::new();
    let mut manual_instructions: Vec<String> = Vec::new();

    // 清理上一个 profile 的配置（镜像源 + 环境变量）
    if let Err(e) = mirror::reset_all_mirrors() {
        errors.push(format!("清除镜像: {e}"));
    }
    if let Err(e) = env::clear_shell_env_block() {
        errors.push(format!("清除环境变量: {e}"));
    }

    // 重新加载配置，确保包含 reset_all_mirrors 清理后的状态
    // （reset_all_mirrors 通过 mutate_current_profile 清除了磁盘上的镜像记录，
    //  但内存中的 cfg 仍持有旧 profile 的镜像数据，必须刷新后再保存）
    cfg = config::AppConfig::load()?;

    // 应用合并后的 profile 配置
    if let Err(e) = env::apply_env_vars(&effective_profile.env) {
        errors.push(format!("环境变量: {e}"));
    }
    match mirror::apply_mirrors(&effective_profile.mirrors) {
        Ok(instructions) => manual_instructions.extend(instructions),
        Err(e) => errors.push(format!("镜像源: {e}")),
    }
    if let Err(e) = proxy::apply_proxy(&effective_profile.proxy) {
        errors.push(format!("代理: {e}"));
    }
    if let Err(e) = git::apply_git_config(&effective_profile.git) {
        errors.push(format!("Git: {e}"));
    }
    match dns::apply_dns(&effective_profile.dns) {
        Ok(Some(instruction)) => manual_instructions.push(instruction),
        Ok(None) => {}
        Err(e) => errors.push(format!("DNS: {e}")),
    }

    // 始终保存配置，避免进程内修改与持久化状态不一致
    cfg.current_profile = profile_name.to_string();
    let config_path = config::config_path()?;
    cfg.save(&config_path)?;

    Ok(SwitchResult { errors, manual_instructions })
}

/// 预览切换 profile 会产生的变更（dry-run）
pub fn preview_switch(profile_name: &str) -> anyhow::Result<()> {
    let cfg = config::AppConfig::load()?;
    let profile = profile::resolve_profile(profile_name)?;

    // 合并项目级配置（如果存在 .nz-switch.toml）
    let effective_profile = match local_config::load_local_config()? {
        Some(ref lc) => {
            println!("  {} 检测到项目级配置 (.nz-switch.toml)，已合并", "📁".blue());
            println!();
            local_config::merge_with_local(&profile, lc, &cfg)?
        }
        None => profile,
    };

    println!("{}", format!("🔍 预览切换到 {} (dry-run)", effective_profile.display_name).cyan().bold());
    println!();

    // 清理操作
    println!("  {}", "将要清理:".bold());
    println!("    • 清除所有现有镜像源配置");
    println!("    • 清除 shell 配置中的 nz-switch 环境变量块");
    println!();

    // 环境变量
    if !effective_profile.env.is_empty() {
        println!("  {}", "将要设置环境变量:".bold());
        for (key, value) in &effective_profile.env {
            println!("    {} = {}", key.cyan(), value);
        }
        println!();
    }

    // 镜像源
    if !effective_profile.mirrors.is_empty() {
        println!("  {}", "将要设置镜像源:".bold());
        for (tool, source) in &effective_profile.mirrors {
            println!("    {} → {}", tool.cyan(), source);
        }
        println!();
    }

    // 代理
    if let Some(proxy) = &effective_profile.proxy {
        println!("  {}", "将要设置代理:".bold());
        println!("    {} ({})", proxy.address.cyan(), proxy.proxy_type);
        println!();
    }

    // Git
    if let Some(git) = &effective_profile.git {
        println!("  {}", "将要设置 Git:".bold());
        if let Some(mirror) = &git.github_mirror {
            println!("    GitHub 镜像: {}", mirror.cyan());
        }
        if let Some(proxy) = &git.proxy {
            println!("    Git 代理: {}", proxy.cyan());
        }
        println!();
    }

    // DNS
    if let Some(dns) = &effective_profile.dns {
        println!("  {}", "将要设置 DNS:".bold());
        println!("    {}", dns.servers.join(", ").cyan());
        println!();
    }

    println!("{} 使用 {} 来实际执行切换", "ℹ".blue(), format!("nz-switch switch {profile_name}").cyan());

    Ok(())
}

/// 获取当前平台的镜像源注册表（配置驱动）
pub fn all_mirror_registries() -> Vec<(String, Vec<(String, String)>)> {
    mirror::config::load_platform_mirrors()
}

/// 获取当前平台信息
pub fn platform_info() -> (String, String) {
    mirror::config::get_platform_info()
}
