use nz-switch::cli::{Cli, Commands, ProfileAction};
use nz-switch::{config, dns, doctor, env, format, local_config, mirror, profile, proxy};

use clap::Parser;
use clap_complete::Shell as CompleteShell;
use colored::Colorize;

fn main() {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    if let Err(e) = run(cli) {
        eprintln!("{} {:#}", "error:".red().bold(), e);
        std::process::exit(1);
    }
}

fn run(cli: Cli) -> anyhow::Result<()> {
    match cli.command {
        Commands::Init { local } => cmd_init(local),
        Commands::Switch { profile, dry_run } => cmd_switch(&profile, dry_run),
        Commands::Status => cmd_status(),
        Commands::Config { action } => cmd_config(action),
        Commands::Mirror { action } => cmd_mirror(action),
        Commands::Proxy { action } => cmd_proxy(action),
        Commands::Env { action } => cmd_env(action),
        Commands::Doctor => cmd_doctor(),
        Commands::Dns { action } => cmd_dns(action),
        Commands::Local { action } => cmd_local(action),
        Commands::Completions { shell } => cmd_completions(shell),
        Commands::Profile { action } => cmd_profile(action),
    }
}

fn cmd_init(local: bool) -> anyhow::Result<()> {
    if local {
        // 初始化项目级配置
        println!("{}", "🔧 初始化项目级配置".cyan().bold());
        println!();

        match local_config::create_local_config() {
            Ok(path) => {
                println!("{} 项目配置文件已创建: {}", "✅".green(), path.display());
                println!();
                println!("编辑 .nz-switch.toml 来覆盖全局配置:");
                println!();
                println!("  {}", "[mirrors]".dimmed());
                println!("  {} = \"https://registry.npmmirror.com\"", "npm".cyan());
                println!();
                println!("  {}", "[env]".dimmed());
                println!("  {} = \"development\"", "NODE_ENV".cyan());
                println!();
                println!("运行 {} 查看当前生效的配置", "nz-switch status".cyan());
            }
            Err(e) => {
                println!("{} {}", "⚠".yellow(), e);
            }
        }
        return Ok(());
    }

    // 全局初始化
    println!("{}", "🔧 nz-switch 初始化".cyan().bold());
    println!();

    let config_path = config::config_path()?;
    if config_path.exists() {
        println!(
            "{} 配置文件已存在: {}",
            "⚠".yellow(),
            config_path.display()
        );
        println!("如需重新初始化，请先删除配置文件。");
        return Ok(());
    }

    let cfg = config::AppConfig::default();
    cfg.save(&config_path)?;

    println!("{} 配置文件已创建: {}", "✅".green(), config_path.display());
    println!();
    println!("接下来你可以:");
    println!("  {} 切换到中国内地环境", "nz-switch switch cn".cyan());
    println!("  {} 切换到海外环境", "nz-switch switch global".cyan());
    println!("  {} 创建项目级配置", "nz-switch init --local".cyan());
    println!("  {} 查看当前状态", "nz-switch status".cyan());
    println!("  {} 测试镜像源速度", "nz-switch mirror test".cyan());

    Ok(())
}

fn cmd_switch(profile_name: &str, dry_run: bool) -> anyhow::Result<()> {
    let profile = profile::resolve_profile(profile_name)?;

    if dry_run {
        return nz-switch::preview_switch(profile_name);
    }

    println!(
        "{} 切换到 {} 环境...",
        "🔄".cyan(),
        profile.display_name.bold()
    );

    let result = nz-switch::switch_profile(profile_name)?;

    for err in &result.errors {
        println!("  ⚠ {err}");
    }

    if result.errors.is_empty() {
        println!("{} 已切换到 {} 环境", "✅".green(), profile.display_name.bold());
    } else {
        println!("{} 已切换到 {} 环境 (部分失败)", "⚠".yellow(), profile.display_name.bold());
    }

    if !result.manual_instructions.is_empty() {
        println!();
        println!("  {} {} 项操作需要手动执行:", "⚠".yellow(), result.manual_instructions.len());
        for instruction in &result.manual_instructions {
            for line in instruction.lines() {
                println!("    {line}");
            }
            println!();
        }
    }

    // 提示项目级配置已自动合并
    if local_config::load_local_config()?.is_some() {
        println!();
        println!(
            "  {} 项目级配置 (.nz-switch.toml) 已自动合并",
            "📁".blue()
        );
    }

    Ok(())
}

fn cmd_status() -> anyhow::Result<()> {
    let cfg = config::AppConfig::load()?;
    let profile = profile::resolve_profile(&cfg.current_profile)?;

    // 检查是否有项目级配置
    let local = local_config::load_local_config()?;
    let effective_profile = match &local {
        Some(lc) => local_config::merge_with_local(&profile, lc, &cfg)?,
        None => profile.clone(),
    };

    println!("{}", "📊 当前环境状态".cyan().bold());
    println!();

    println!("  {} {}", "Profile:".bold(), profile.display_name);

    // 显示项目级配置覆盖
    if let Some(lc) = &local {
        if let Some(path) = local_config::find_local_config() {
            println!("  {} {} (已合并)", "项目配置:".bold(), path.display());
        }
        if !lc.env.is_empty() || !lc.mirrors.is_empty() {
            println!("    {} 项目级覆盖:", "⚡".yellow());
            for key in lc.env.keys() {
                println!("      env.{} = {}", key.cyan(), lc.env[key]);
            }
            for key in lc.mirrors.keys() {
                println!("      mirrors.{} = {}", key.cyan(), lc.mirrors[key]);
            }
        }
    }

    println!();
    println!("  {}", "环境变量 (生效值):".bold());
    if effective_profile.env.is_empty() {
        println!("    (无自定义环境变量)");
    } else {
        let mut table = format::new_table(&["变量", "值"]);
        for (key, value) in &effective_profile.env {
            table.add_row(vec![key.cyan().to_string(), value.to_string()]);
        }
        println!("{table}");
    }

    println!();
    println!("  {}", "镜像源 (生效值):".bold());
    if effective_profile.mirrors.is_empty() {
        println!("    (无自定义镜像源)");
    } else {
        let mut table = format::new_table(&["工具", "当前镜像源"]);
        for (tool, source) in &effective_profile.mirrors {
            table.add_row(vec![tool.cyan().to_string(), source.to_string()]);
        }
        println!("{table}");
    }

    println!();
    println!("  {}", "代理:".bold());
    match &effective_profile.proxy {
        Some(p) => {
            println!("    地址: {}", p.address.cyan());
            println!("    类型: {}", p.proxy_type);
        }
        None => println!("    (未配置代理)"),
    }

    println!();
    println!("  {}", "Git:".bold());
    match &effective_profile.git {
        Some(g) => {
            if let Some(m) = &g.github_mirror {
                println!("    GitHub 镜像: {}", m.cyan());
            }
            if let Some(p) = &g.proxy {
                println!("    Git 代理: {}", p.cyan());
            }
        }
        None => println!("    (无自定义 Git 配置)"),
    }

    println!();
    println!("  {}", "DNS:".bold());
    match &effective_profile.dns {
        Some(d) => {
            println!("    服务器: {}", d.servers.join(", ").cyan());
        }
        None => println!("    (未配置 DNS)"),
    }

    Ok(())
}

fn cmd_config(action: nz-switch::cli::ConfigAction) -> anyhow::Result<()> {
    match action {
        nz-switch::cli::ConfigAction::Show => {
            let cfg = config::AppConfig::load()?;
            println!("{}", "⚙️  当前配置".cyan().bold());
            println!();
            println!("{}", toml::to_string_pretty(&cfg)?);
        }
        nz-switch::cli::ConfigAction::Path => {
            let path = config::config_path()?;
            println!("{}", path.display());
        }
        nz-switch::cli::ConfigAction::Edit => {
            let path = config::config_path()?;
            let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vi".to_string());
            println!("{} 打开配置文件: {}", "📝".cyan(), path.display());
            std::process::Command::new(&editor)
                .arg(&path)
                .status()?;
        }
        nz-switch::cli::ConfigAction::Export { output } => {
            let cfg = config::AppConfig::load()?;
            let json = serde_json::to_string_pretty(&cfg)?;

            match output {
                Some(path) => {
                    std::fs::write(&path, &json)?;
                    println!("{} 配置已导出到: {}", "✅".green(), path.cyan());
                }
                None => {
                    println!("{json}");
                }
            }
        }
        nz-switch::cli::ConfigAction::Import { file, merge } => {
            let content = std::fs::read_to_string(&file)?;
            let imported: config::AppConfig = if file.ends_with(".json") {
                serde_json::from_str(&content)?
            } else {
                toml::from_str(&content)?
            };

            let config_path = config::config_path()?;

            if merge {
                let mut cfg = config::AppConfig::load()?;
                // 合并 profiles
                for (name, profile) in imported.profiles {
                    cfg.profiles.insert(name, profile);
                }
                cfg.save(&config_path)?;
                println!("{} 配置已合并导入", "✅".green());
            } else {
                imported.save(&config_path)?;
                println!("{} 配置已替换导入", "✅".green());
            }

            println!("  配置文件: {}", config_path.display());
        }
        nz-switch::cli::ConfigAction::Reset => {
            let config_path = config::config_path()?;
            let cfg = config::AppConfig::default();
            cfg.save(&config_path)?;
            println!("{} 配置已重置为默认值", "✅".green());
            println!("  配置文件: {}", config_path.display());
        }
    }
    Ok(())
}

fn cmd_mirror(action: nz-switch::cli::MirrorAction) -> anyhow::Result<()> {
    match action {
        nz-switch::cli::MirrorAction::List { tool } => mirror::list_mirrors(tool.as_deref()),
        nz-switch::cli::MirrorAction::Test { tool } => mirror::test_mirrors(tool.as_deref()),
        nz-switch::cli::MirrorAction::Set { tool, source } => mirror::set_mirror(&tool, &source),
        nz-switch::cli::MirrorAction::Reset { tool } => mirror::reset_mirror(&tool),
    }
}

fn cmd_proxy(action: nz-switch::cli::ProxyAction) -> anyhow::Result<()> {
    match action {
        nz-switch::cli::ProxyAction::Set { address } => proxy::set_proxy(&address),
        nz-switch::cli::ProxyAction::On => proxy::enable_proxy(),
        nz-switch::cli::ProxyAction::Off => proxy::disable_proxy(),
        nz-switch::cli::ProxyAction::Test => proxy::test_proxy(),
    }
}

fn cmd_env(action: nz-switch::cli::EnvAction) -> anyhow::Result<()> {
    match action {
        nz-switch::cli::EnvAction::Show => env::show_env(),
        nz-switch::cli::EnvAction::Set { key, value } => env::set_env(&key, &value),
        nz-switch::cli::EnvAction::Unset { key } => env::unset_env(&key),
        nz-switch::cli::EnvAction::Proxy => env::show_proxy_env(),
    }
}

fn cmd_local(action: nz-switch::cli::LocalAction) -> anyhow::Result<()> {
    match action {
        nz-switch::cli::LocalAction::Show => local_config::show_local_config(),
        nz-switch::cli::LocalAction::Init => {
            match local_config::create_local_config() {
                Ok(path) => println!("{} 项目配置文件已创建: {}", "✅".green(), path.display()),
                Err(e) => println!("{} {}", "⚠".yellow(), e),
            }
            Ok(())
        }
        nz-switch::cli::LocalAction::Path => {
            match local_config::find_local_config() {
                Some(path) => println!("{}", path.display()),
                None => println!("(未找到项目配置文件)"),
            }
            Ok(())
        }
    }
}

fn cmd_dns(action: nz-switch::cli::DnsAction) -> anyhow::Result<()> {
    match action {
        nz-switch::cli::DnsAction::Show => dns::show_dns(),
        nz-switch::cli::DnsAction::List => dns::list_dns_presets(),
        nz-switch::cli::DnsAction::Set { source } => dns::set_dns(&source),
    }
}

fn cmd_doctor() -> anyhow::Result<()> {
    doctor::run_diagnosis()
}

fn cmd_profile(action: ProfileAction) -> anyhow::Result<()> {
    match action {
        ProfileAction::List => {
            let cfg = config::AppConfig::load()?;
            println!("{}", "📋 Profile 列表".cyan().bold());
            println!();

            let mut table = format::new_table(&["Profile", "名称", "类型", "状态"]);
            for (name, p) in &cfg.profiles {
                let is_current = name == &cfg.current_profile;
                let status = if is_current { "← 当前".green().to_string() } else { String::new() };
                let is_builtin = name == "cn" || name == "global";
                let tag = if is_builtin { "内置".dimmed().to_string() } else { "自定义".blue().to_string() };
                table.add_row(vec![name.bold().to_string(), p.display_name.clone(), tag, status]);
            }
            println!("{table}");

            println!();
            println!("运行 {} 切换 profile", "nz-switch switch <name>".cyan());
        }
        ProfileAction::Create { name } => {
            if name == "cn" || name == "global" {
                anyhow::bail!("不能覆盖内置 profile '{name}'");
            }

            let mut cfg = config::AppConfig::load()?;
            if cfg.profiles.contains_key(&name) {
                anyhow::bail!("profile '{name}' 已存在");
            }

            let new_profile = profile::Profile {
                display_name: name.clone(),
                env: std::collections::HashMap::new(),
                mirrors: std::collections::HashMap::new(),
                proxy: None,
                git: None,
                dns: None,
            };
            cfg.profiles.insert(name.clone(), new_profile);
            let config_path = config::config_path()?;
            cfg.save(&config_path)?;

            println!("{} Profile '{}' 已创建", "✅".green(), name.cyan());
            println!("  运行 {} 开始配置", format!("nz-switch switch {name}").cyan());
        }
        ProfileAction::Delete { name } => {
            if name == "cn" || name == "global" {
                anyhow::bail!("不能删除内置 profile '{name}'");
            }

            let mut cfg = config::AppConfig::load()?;
            if !cfg.profiles.contains_key(&name) {
                anyhow::bail!("profile '{name}' 不存在");
            }

            if cfg.current_profile == name {
                anyhow::bail!("不能删除当前正在使用的 profile '{name}'，请先切换到其他 profile");
            }

            cfg.profiles.remove(&name);
            let config_path = config::config_path()?;
            cfg.save(&config_path)?;

            println!("{} Profile '{}' 已删除", "✅".green(), name.cyan());
        }
    }

    Ok(())
}

fn cmd_completions(shell: nz-switch::cli::Shell) -> anyhow::Result<()> {
    let complete_shell = match shell {
        nz-switch::cli::Shell::Bash => CompleteShell::Bash,
        nz-switch::cli::Shell::Zsh => CompleteShell::Zsh,
        nz-switch::cli::Shell::Fish => CompleteShell::Fish,
        nz-switch::cli::Shell::PowerShell => CompleteShell::PowerShell,
        nz-switch::cli::Shell::Elvish => CompleteShell::Elvish,
    };

    let mut cmd = <Cli as clap::CommandFactory>::command();
    clap_complete::generate(
        complete_shell,
        &mut cmd,
        "nz-switch",
        &mut std::io::stdout(),
    );

    eprintln!();
    match shell {
        nz-switch::cli::Shell::Bash => {
            eprintln!("{} 安装到 bash:", "💡".green());
            eprintln!("  nz-switch completions bash > ~/.local/share/bash-completion/completions/nz-switch");
        }
        nz-switch::cli::Shell::Zsh => {
            eprintln!("{} 安装到 zsh:", "💡".green());
            eprintln!("  nz-switch completions zsh > ~/.zfunc/_nz-switch");
        }
        nz-switch::cli::Shell::Fish => {
            eprintln!("{} 安装到 fish:", "💡".green());
            eprintln!("  nz-switch completions fish > ~/.config/fish/completions/nz-switch.fish");
        }
        nz-switch::cli::Shell::PowerShell => {
            eprintln!("{} 安装到 PowerShell:", "💡".green());
            eprintln!("  nz-switch completions powershell | Out-String | Invoke-Expression");
        }
        nz-switch::cli::Shell::Elvish => {
            eprintln!("{} 安装到 elvish:", "💡".green());
            eprintln!("  nz-switch completions elvish > ~/.config/elvish/completions/nz-switch.elv");
        }
    }

    Ok(())
}
