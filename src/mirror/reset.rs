// 镜像源重置逻辑

use anyhow::Result;
use colored::Colorize;

use super::config;
use super::env_vars;
use super::paths;

/// 重置某个工具的镜像源为默认（同时清除工具配置文件和 nz-switch 配置）
pub fn reset_mirror(tool: &str) -> Result<()> {
    // 先尝试清理工具配置文件，再保存 nz-switch 配置（避免文件操作失败导致半重置状态）
    match tool {
        "pip" => {
            let conf_path = paths::pip_config_path()?;
            if conf_path.exists() {
                std::fs::remove_file(&conf_path)?;
                println!("{} 已删除 {}", "✅".green(), conf_path.display());
            } else {
                println!("{} pip 配置文件不存在", "ℹ".blue());
            }
        }
        "npm" | "pnpm" => {
            let npmrc = paths::npmrc_path()?;

            if npmrc.exists() {
                let content = std::fs::read_to_string(&npmrc)?;
                let new_content: String = content
                    .lines()
                    .filter(|line| !line.starts_with("registry="))
                    .collect::<Vec<_>>()
                    .join("\n");
                std::fs::write(&npmrc, new_content)?;
                println!("{} 已从 .npmrc 移除 registry 配置", "✅".green());
            } else {
                println!("{} npm/pnpm 配置文件不存在", "ℹ".blue());
            }
        }
        "yarn" => {
            let yarnrc = paths::yarnrc_path()?;

            if yarnrc.exists() {
                let content = std::fs::read_to_string(&yarnrc)?;
                let new_content: String = content
                    .lines()
                    .filter(|line| !line.trim().starts_with("registry "))
                    .collect::<Vec<_>>()
                    .join("\n");
                std::fs::write(&yarnrc, new_content)?;
                println!("{} 已从 .yarnrc 移除 registry 配置", "✅".green());
            } else {
                println!("{} yarn 配置文件不存在", "ℹ".blue());
            }
        }
        "cargo" => {
            let config_path = paths::cargo_config_path()?;
            if config_path.exists() {
                std::fs::remove_file(&config_path)?;
                println!("{} 已删除 {}", "✅".green(), config_path.display());
            } else {
                println!("{} cargo 配置文件不存在", "ℹ".blue());
            }
        }
        "conda" => {
            let condarc = paths::condarc_path()?;
            if condarc.exists() {
                std::fs::remove_file(&condarc)?;
                println!("{} 已删除 {}", "✅".green(), condarc.display());
            } else {
                println!("{} conda 配置文件不存在", "ℹ".blue());
            }
        }
        "rubygems" => {
            println!("    {} rubygems 镜像重置请手动执行:", "⚠".yellow());
            println!("      gem sources --remove <mirror-url>");
            println!("      gem sources --add https://rubygems.org/");
        }
        "composer" => {
            println!("    {} composer 镜像重置请手动执行:", "⚠".yellow());
            println!("      composer config -g repo.packagist composer https://packagist.org");
        }
        "docker" => {
            let home_config = paths::docker_user_daemon_path().ok();
            if let Some(path) = home_config {
                if path.exists() {
                    let content = std::fs::read_to_string(&path)?;
                    let mut config_val: serde_json::Value =
                        serde_json::from_str(&content).unwrap_or(serde_json::json!({}));
                    if config_val.get("registry-mirrors").is_some() {
                        config_val
                            .as_object_mut()
                            .map(|m| m.remove("registry-mirrors"));
                        if let Ok(new_content) = serde_json::to_string_pretty(&config_val) {
                            std::fs::write(&path, new_content)?;
                        }
                    }
                    println!(
                        "{} 已从 {} 移除 registry-mirrors",
                        "✅".green(),
                        path.display()
                    );
                } else {
                    println!("{} Docker 用户配置文件不存在", "ℹ".blue());
                }
            }
        }
        "pacman" => {
            let mirrorlist = std::path::Path::new("/etc/pacman.d/mirrorlist");
            let pacman_conf = std::path::Path::new("/etc/pacman.conf");

            // 重置 mirrorlist 为注释状态
            if mirrorlist.exists() {
                let content = std::fs::read_to_string(mirrorlist)?;
                if content.contains("nz-switch") {
                    // 只清除我们写入的内容，保留其他
                    let new_content: String = content
                        .lines()
                        .filter(|line| !line.contains("nz-switch"))
                        .collect::<Vec<_>>()
                        .join("\n");
                    std::fs::write(mirrorlist, new_content)?;
                    println!(
                        "{} 已清理 /etc/pacman.d/mirrorlist 中的 nz-switch 配置",
                        "✅".green()
                    );
                } else {
                    println!("{} mirrorlist 非 nz-switch 创建，跳过", "ℹ".blue());
                }
            }

            // 从 pacman.conf 中移除 [archlinuxcn] section
            if pacman_conf.exists() {
                let content = std::fs::read_to_string(pacman_conf)?;
                if content.contains("[archlinuxcn]") && content.contains("nz-switch") {
                    let new_content: String = content
                        .lines()
                        .collect::<Vec<_>>()
                        .split(|line| line.trim() == "[archlinuxcn]")
                        .next()
                        .unwrap_or(&[])
                        .iter()
                        .map(|s| s.to_string())
                        .collect::<Vec<_>>()
                        .join("\n");
                    std::fs::write(pacman_conf, new_content.trim_end().to_string() + "\n")?;
                    println!(
                        "{} 已从 pacman.conf 移除 [archlinuxcn] section",
                        "✅".green()
                    );
                }
            }
        }
        "gradle" => {
            let init_path = paths::gradle_init_path()?;
            if init_path.exists() {
                let content = std::fs::read_to_string(&init_path)?;
                if content.contains("nz-switch") {
                    std::fs::remove_file(&init_path)?;
                    println!("{} 已删除 {}", "✅".green(), init_path.display());
                } else {
                    println!("{} init.gradle 非 nz-switch 创建，跳过删除", "ℹ".blue());
                }
            } else {
                println!("{} gradle init.gradle 不存在", "ℹ".blue());
            }
        }
        _ => {
            // 尝试通过 env_vars 清除环境变量
            let var_names = env_vars::env_var_names(tool);
            if !var_names.is_empty() {
                for var_name in var_names {
                    // SAFETY: 单线程上下文调用
                    unsafe {
                        std::env::remove_var(var_name);
                    }
                    crate::env::remove_from_shell_config(var_name)?;
                }
                println!("{} 已清除 {} 的环境变量", "✅".green(), tool);
            } else {
                println!("{} 暂不支持重置 {} 的镜像源", "⚠".yellow(), tool);
            }
        }
    }

    // 工具配置文件清理成功后，从 nz-switch 配置中移除镜像记录
    let tool = tool.to_string();
    let _ = crate::config::mutate_current_profile(|profile| {
        profile.mirrors.remove(&tool);
    });
    // profile 不存在时不报错（重置操作本身就是要清除）

    Ok(())
}

/// 重置所有工具的镜像源为默认 (用于 switch global)
pub fn reset_all_mirrors() -> Result<()> {
    println!("  {}", "清除国内镜像配置...".bold());

    // 重置所有当前平台支持的工具
    let platform_mirrors = config::load_platform_mirrors();
    for (tool, _mirrors) in &platform_mirrors {
        if let Err(e) = reset_mirror(tool) {
            tracing::warn!("重置 {} 镜像失败: {}", tool, e);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_npm_reset_removes_registry_line() {
        let dir = std::env::temp_dir().join("nz-switch-test-reset-npm");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        let npmrc = dir.join(".npmrc");
        std::fs::write(
            &npmrc,
            "registry=https://registry.npmmirror.com/\nother=value\n",
        )
        .unwrap();

        // 模拟 reset 逻辑
        let content = std::fs::read_to_string(&npmrc).unwrap();
        let new_content: String = content
            .lines()
            .filter(|line| !line.starts_with("registry="))
            .collect::<Vec<_>>()
            .join("\n");
        std::fs::write(&npmrc, new_content).unwrap();

        let result = std::fs::read_to_string(&npmrc).unwrap();
        assert!(!result.contains("registry="));
        assert!(result.contains("other=value"));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_yarn_reset_removes_registry_line() {
        let dir = std::env::temp_dir().join("nz-switch-test-reset-yarn");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        let yarnrc = dir.join(".yarnrc");
        std::fs::write(
            &yarnrc,
            "registry \"https://registry.npmmirror.com/\"\nother-config true\n",
        )
        .unwrap();

        // 模拟 reset 逻辑
        let content = std::fs::read_to_string(&yarnrc).unwrap();
        let new_content: String = content
            .lines()
            .filter(|line| !line.trim().starts_with("registry "))
            .collect::<Vec<_>>()
            .join("\n");
        std::fs::write(&yarnrc, new_content).unwrap();

        let result = std::fs::read_to_string(&yarnrc).unwrap();
        assert!(!result.contains("registry "));
        assert!(result.contains("other-config true"));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_pnpm_reset_uses_npmrc() {
        // pnpm 和 npm 共用 .npmrc 文件
        let dir = std::env::temp_dir().join("nz-switch-test-reset-pnpm");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        let npmrc = dir.join(".npmrc");
        std::fs::write(
            &npmrc,
            "registry=https://registry.npmmirror.com/\nstore-dir=.pnpm-store\n",
        )
        .unwrap();

        let content = std::fs::read_to_string(&npmrc).unwrap();
        let new_content: String = content
            .lines()
            .filter(|line| !line.starts_with("registry="))
            .collect::<Vec<_>>()
            .join("\n");
        std::fs::write(&npmrc, new_content).unwrap();

        let result = std::fs::read_to_string(&npmrc).unwrap();
        assert!(!result.contains("registry="));
        assert!(result.contains("store-dir=.pnpm-store"));

        let _ = std::fs::remove_dir_all(&dir);
    }
}
