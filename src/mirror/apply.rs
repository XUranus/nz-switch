// 镜像源应用逻辑

use anyhow::Result;
use colored::Colorize;
use std::collections::HashMap;
use tracing::info;

use super::config;
use super::env_vars;
use super::paths;
use super::types::ApplyResult;

/// 应用镜像源配置，返回手动操作提示列表
pub fn apply_mirrors(mirrors: &HashMap<String, String>) -> Result<Vec<String>> {
    let mut manual_instructions: Vec<String> = Vec::new();

    if mirrors.is_empty() {
        println!("  {} 镜像源: 无自定义配置", "ℹ".blue());
        return Ok(manual_instructions);
    }

    println!("  {}", "镜像源:".bold());

    for (tool, source) in mirrors {
        match apply_single_mirror(tool, source) {
            Ok(ApplyResult::Applied) => {
                println!("    {} {} → {}", "✓".green(), tool.cyan(), source);
            }
            Ok(ApplyResult::ManualRequired(instruction)) => {
                println!(
                    "    {} {} → {} (需手动配置)",
                    "⚠".yellow(),
                    tool.cyan(),
                    source
                );
                manual_instructions.push(instruction);
            }
            Err(e) => {
                println!("    {} {} → {} ({})", "✗".red(), tool.cyan(), source, e);
            }
        }
    }

    Ok(manual_instructions)
}

/// 应用单个镜像源（根据 config_type 自动分派）
pub fn apply_single_mirror(tool: &str, source: &str) -> Result<ApplyResult> {
    // 查找镜像定义以获取 config_type
    let mirror_def = config::find_mirror_def(tool);

    let config_type = mirror_def
        .as_ref()
        .map(|d| d.config_type.as_str())
        .unwrap_or("manual");

    match config_type {
        "file" => apply_file_mirror(tool, source),
        "env" => apply_env_mirror(tool, source),
        _ => apply_manual_mirror(tool, source),
    }
}

/// 设置某个工具的镜像源（同时写入工具配置文件和 nz-switch 配置）
pub fn set_mirror(tool: &str, source: &str) -> Result<()> {
    apply_single_mirror(tool, source)?;

    // 同步到 nz-switch 配置
    let tool_owned = tool.to_string();
    let source_owned = source.to_string();
    crate::config::mutate_current_profile(|profile| {
        profile
            .mirrors
            .insert(tool_owned.clone(), source_owned.clone());
    })?;

    println!(
        "{} {} 镜像源已设置为: {}",
        "✅".green(),
        tool.cyan(),
        source
    );
    Ok(())
}

/// file 类型：写入工具配置文件
fn apply_file_mirror(tool: &str, source: &str) -> Result<ApplyResult> {
    // 统一将镜像名解析为 URL
    let url = config::resolve_mirror_url(tool, source)?;
    match tool {
        "pip" => apply_pip_mirror(&url),
        "npm" => apply_npm_mirror(&url),
        "yarn" => apply_yarn_mirror(&url),
        "pnpm" => apply_pnpm_mirror(&url),
        "cargo" => apply_cargo_mirror(&url),
        "conda" => apply_conda_mirror(&url),
        "maven" => apply_maven_mirror(&url),
        "gradle" => apply_gradle_mirror(&url),
        "docker" => apply_docker_mirror(&url),
        "pacman" => apply_pacman_mirror(source),
        _ => {
            let msg = format!("暂不支持自动配置 {tool} 的镜像源文件，请参考镜像站文档手动配置");
            info!("no file handler for tool: {}, skipping", tool);
            Ok(ApplyResult::ManualRequired(msg))
        }
    }
}

/// env 类型：解析预设 URL 并写入 shell 配置
fn apply_env_mirror(tool: &str, source: &str) -> Result<ApplyResult> {
    let entries = env_vars::env_var_entries(tool, source)?;
    if entries.is_empty() {
        let msg = format!("暂不支持 {tool} 的镜像源: {source}");
        println!("    {} {}", "⚠".yellow(), msg);
        return Ok(ApplyResult::ManualRequired(msg));
    }

    for (key, value) in &entries {
        crate::env::set_env(key, value)?;
        println!("    {} {} = {}", "✓".green(), key.cyan(), value);
    }
    Ok(ApplyResult::Applied)
}

/// manual 类型：打印手动操作提示
fn apply_manual_mirror(tool: &str, source: &str) -> Result<ApplyResult> {
    match tool {
        "choco" => apply_choco_mirror(source),
        "nuget" => apply_nuget_mirror(source),
        "rubygems" => apply_rubygems_mirror(source),
        "composer" => apply_composer_mirror(source),
        "python" => {
            let url =
                config::resolve_mirror_url("python", source).unwrap_or_else(|_| source.to_string());
            let msg = format!("Python 安装包镜像: {url}，请从以上地址下载");
            println!("    {} {}", "ℹ".blue(), msg);
            Ok(ApplyResult::ManualRequired(msg))
        }
        "cocoapods" => apply_cocoapods_mirror(source),
        "vscode" => apply_vscode_mirror(source),
        "android-maven" => apply_android_maven_mirror(source),
        "android-gradle" => apply_android_gradle_mirror(source),
        "swift" => apply_swift_mirror(source),
        "k8s-gcr" | "k8s-registry" | "ghcr" | "quay" => {
            let msg = format!("{tool} 镜像需要替换 YAML 中的镜像地址，请参考对应镜像站文档");
            println!("    {} {}", "⚠".yellow(), msg);
            Ok(ApplyResult::ManualRequired(msg))
        }
        _ => {
            let msg = format!("暂不支持自动配置 {tool} 的镜像源，请参考镜像站文档手动配置");
            info!("no manual handler for tool: {}, skipping", tool);
            Ok(ApplyResult::ManualRequired(msg))
        }
    }
}

// ─── 各工具镜像源写入 ────────────────────────────────────────────────

/// 应用 pip 镜像
fn apply_pip_mirror(url: &str) -> Result<ApplyResult> {
    let conf_path = paths::pip_config_path()?;

    if let Some(parent) = conf_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // 确保 URL 以 /simple 结尾（PEP 503 规范）
    let index_url = if url.ends_with("/simple") || url.ends_with("/simple/") {
        url.to_string()
    } else if url.ends_with('/') {
        format!("{url}simple")
    } else {
        format!("{url}/simple")
    };

    // 提取 hostname 作为 trusted-host
    let trusted_host = index_url
        .replace("https://", "")
        .replace("http://", "")
        .split('/')
        .next()
        .unwrap_or(&index_url)
        .to_string();

    let content = format!("[global]\nindex-url = {index_url}\ntrusted-host = {trusted_host}\n");

    std::fs::write(&conf_path, content)?;
    info!("wrote pip config to {}", conf_path.display());

    Ok(ApplyResult::Applied)
}

/// 应用 npm 镜像
fn apply_npm_mirror(url: &str) -> Result<ApplyResult> {
    let npmrc = paths::npmrc_path()?;

    let content = if npmrc.exists() {
        std::fs::read_to_string(&npmrc)?
    } else {
        String::new()
    };

    let new_content = if content.contains("registry=") {
        content
            .lines()
            .map(|line| {
                if line.starts_with("registry=") {
                    format!("registry={url}")
                } else {
                    line.to_string()
                }
            })
            .collect::<Vec<_>>()
            .join("\n")
    } else {
        let mut c = content;
        if !c.ends_with('\n') && !c.is_empty() {
            c.push('\n');
        }
        c.push_str(&format!("registry={url}\n"));
        c
    };

    std::fs::write(&npmrc, new_content)?;
    info!("wrote npm config to {}", npmrc.display());

    Ok(ApplyResult::Applied)
}

/// 应用 cargo 镜像
fn apply_cargo_mirror(source: &str) -> Result<ApplyResult> {
    let config_path = paths::cargo_config_path()?;

    if let Some(parent) = config_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // 从镜像配置中查找 URL
    let registry_url = config::resolve_mirror_url("cargo", source)?;

    // .git URL 使用 git 协议，其他使用 sparse 协议
    let registry_value = if registry_url.ends_with(".git") {
        format!("git+{registry_url}")
    } else {
        // 确保非 git URL 以 / 结尾
        let url = if registry_url.ends_with('/') {
            registry_url.clone()
        } else {
            format!("{registry_url}/")
        };
        format!("sparse+{url}")
    };

    let content = format!(
        "[source.crates-io]\nreplace-with = 'mirror'\n\n[source.mirror]\nregistry = \"{registry_value}\"\n"
    );

    std::fs::write(&config_path, content)?;
    info!("wrote cargo config to {}", config_path.display());

    Ok(ApplyResult::Applied)
}

/// 应用 Docker 镜像
/// 先尝试写入 ~/.docker/daemon.json（用户级，不需 sudo）
/// 如果不可用，回退到手动提示
fn apply_docker_mirror(url: &str) -> Result<ApplyResult> {
    // 尝试用户级 Docker 配置目录
    if let Ok(daemon_json_path) = paths::docker_user_daemon_path() {
        if let Some(docker_dir) = daemon_json_path.parent() {
            if std::fs::create_dir_all(docker_dir).is_ok() {
                // 读取现有配置或创建新配置
                let mut config: serde_json::Value = if daemon_json_path.exists() {
                    let content = std::fs::read_to_string(&daemon_json_path)?;
                    serde_json::from_str(&content).unwrap_or(serde_json::json!({}))
                } else {
                    serde_json::json!({})
                };

                // 设置 registry-mirrors
                config["registry-mirrors"] = serde_json::json!([url]);

                if let Ok(content) = serde_json::to_string_pretty(&config) {
                    if std::fs::write(&daemon_json_path, content).is_ok() {
                        info!("wrote docker config to {}", daemon_json_path.display());
                        println!("    {} 已写入 {}", "✓".green(), daemon_json_path.display());
                        println!("    {} 请重启 Docker daemon 使配置生效", "ℹ".blue());
                        return Ok(ApplyResult::Applied);
                    }
                }
            }
        }
    }

    // 回退: 手动提示
    let sys_path = paths::docker_sys_daemon_path();
    let instruction = format!(
        "sudo tee {} <<-'EOF'\n{{\"registry-mirrors\": [\"{}\"]}}\nEOF\nsudo systemctl restart docker",
        sys_path.display(), url
    );
    println!(
        "    {} Docker 镜像需要 sudo 权限，请手动执行:",
        "⚠".yellow()
    );
    println!(
        "      {}",
        instruction.lines().collect::<Vec<_>>().join("\n      ")
    );
    Ok(ApplyResult::ManualRequired(instruction))
}

/// 应用 Conda 镜像
fn apply_conda_mirror(source: &str) -> Result<ApplyResult> {
    let condarc = paths::condarc_path()?;

    let base_url =
        config::resolve_mirror_url("conda", source).unwrap_or_else(|_| source.to_string());

    // 如果已有 .condarc 且非我们写的，备份
    if condarc.exists() {
        let existing = std::fs::read_to_string(&condarc)?;
        if !existing.is_empty() && !existing.contains("nz-switch") {
            let backup_path = crate::home_dir()?.join(".condarc.bak");
            std::fs::write(&backup_path, &existing)?;
            info!("backed up existing .condarc to {}", backup_path.display());
        }
    }

    let content = format!(
        "# nz-switch mirror config\nchannels:\n  - defaults\nshow_channel_urls: true\ndefault_channels:\n  - {base_url}/pkgs/main\n  - {base_url}/pkgs/r\n  - {base_url}/pkgs/msys2\ncustom_channels:\n  conda-forge: {base_url}/cloud\n"
    );

    std::fs::write(&condarc, content)?;
    info!("wrote conda config to {}", condarc.display());

    Ok(ApplyResult::Applied)
}

/// 应用 pacman 镜像 (主仓库 + archlinuxcn)
fn apply_pacman_mirror(source: &str) -> Result<ApplyResult> {
    use std::path::Path;

    let mirrorlist = Path::new("/etc/pacman.d/mirrorlist");
    let pacman_conf = Path::new("/etc/pacman.conf");

    // 解析主仓库 URL
    let main_url =
        config::resolve_mirror_url("pacman", source).unwrap_or_else(|_| source.to_string());

    // 获取 archlinuxcn URL（从 mirror def 的 env_vars 中读取）
    let cn_url = config::find_mirror_def("pacman")
        .and_then(|def| {
            def.mirrors
                .iter()
                .find(|m| m.name == source)
                .and_then(|m| m.env_vars.as_ref())
                .and_then(|vars| vars.get("archlinuxcn").cloned())
        })
        .unwrap_or_else(|| {
            // 从主 URL 推导: .../archlinux/$repo/os/$arch → .../archlinuxcn/$arch
            main_url
                .replace("/archlinux/$repo/os/$arch", "/archlinuxcn/$arch")
                .replace("/archlinux/$repo/", "/archlinuxcn/")
        });

    // 写入 /etc/pacman.d/mirrorlist
    let mirrorlist_content = format!("# nz-switch mirror config\nServer = {main_url}\n");
    std::fs::write(mirrorlist, &mirrorlist_content)?;
    info!("wrote pacman mirrorlist to {}", mirrorlist.display());

    // 写入 /etc/pacman.conf 中的 [archlinuxcn] section
    if pacman_conf.exists() {
        let existing = std::fs::read_to_string(pacman_conf)?;
        let new_content = if existing.contains("[archlinuxcn]") {
            // 替换已有的 Server 行
            existing
                .lines()
                .map(|line| {
                    if line.trim_start().starts_with("Server")
                        && existing[..existing.find(line).unwrap_or(0)].contains("[archlinuxcn]")
                    {
                        // 检查此 Server 行是否在 [archlinuxcn] section 内
                        // 简单策略: 如果上一个非空非注释 section header 是 [archlinuxcn]，则替换
                        format!("Server = {cn_url}")
                    } else {
                        line.to_string()
                    }
                })
                .collect::<Vec<_>>()
                .join("\n")
        } else {
            // 追加 [archlinuxcn] section
            format!("{existing}\n# nz-switch mirror config\n[archlinuxcn]\nServer = {cn_url}\n")
        };
        std::fs::write(pacman_conf, &new_content)?;
        info!("updated archlinuxcn in {}", pacman_conf.display());
    } else {
        // 创建 pacman.conf with archlinuxcn
        let content = format!(
            "# nz-switch mirror config\n[options]\nHoldPkg = pacman glibc\n\n[core]\nInclude = /etc/pacman.d/mirrorlist\n\n[extra]\nInclude = /etc/pacman.d/mirrorlist\n\n[archlinuxcn]\nServer = {cn_url}\n"
        );
        std::fs::write(pacman_conf, content)?;
        info!("created {} with archlinuxcn", pacman_conf.display());
    }

    Ok(ApplyResult::Applied)
}

/// 应用 Chocolatey 镜像 (仅打印提示, Windows)
fn apply_choco_mirror(source: &str) -> Result<ApplyResult> {
    let mirror_url =
        config::resolve_mirror_url("choco", source).unwrap_or_else(|_| source.to_string());

    println!(
        "    {} Chocolatey 镜像需要管理员权限，请在 PowerShell 中执行:",
        "⚠".yellow()
    );
    println!("      choco source add -n=mirror -s='{mirror_url}' --priority=1");
    println!("      choco source remove -n=chocolatey");

    Ok(ApplyResult::ManualRequired(
        "Chocolatey 镜像需要管理员权限，请手动配置".into(),
    ))
}

/// 应用 NuGet 镜像 (仅打印提示，Windows/.NET)
fn apply_nuget_mirror(source: &str) -> Result<ApplyResult> {
    let mirror_url =
        config::resolve_mirror_url("nuget", source).unwrap_or_else(|_| source.to_string());

    println!("    {} NuGet 镜像请在 dotnet 项目中配置:", "⚠".yellow());
    println!("      dotnet nuget add source '{mirror_url}' -n mirror");

    Ok(ApplyResult::ManualRequired(
        "NuGet 镜像请在 dotnet 项目中配置".into(),
    ))
}

/// 应用 Maven 镜像 (仅打印提示，Java)
fn apply_maven_mirror(source: &str) -> Result<ApplyResult> {
    let mirror_url =
        config::resolve_mirror_url("maven", source).unwrap_or_else(|_| source.to_string());

    let settings_path = paths::maven_settings_path()?;

    println!(
        "    {} Maven 镜像请在 {} 中配置:",
        "⚠".yellow(),
        settings_path.display()
    );
    println!("      <mirrors>");
    println!("        <mirror>");
    println!("          <id>mirror</id>");
    println!("          <mirrorOf>central</mirrorOf>");
    println!("          <url>{mirror_url}</url>");
    println!("        </mirror>");
    println!("      </mirrors>");

    Ok(ApplyResult::ManualRequired(
        "Maven 镜像请在 settings.xml 中配置".into(),
    ))
}

/// 应用 RubyGems 镜像 (仅打印提示)
fn apply_rubygems_mirror(source: &str) -> Result<ApplyResult> {
    let mirror_url =
        config::resolve_mirror_url("rubygems", source).unwrap_or_else(|_| source.to_string());

    println!("    {} RubyGems 镜像请手动执行:", "⚠".yellow());
    println!("      gem sources --add {mirror_url}");
    println!("      gem sources --remove https://rubygems.org/");

    Ok(ApplyResult::ManualRequired(
        "RubyGems 镜像请手动执行 gem sources 命令".into(),
    ))
}

/// 应用 Composer 镜像 (仅打印提示，PHP)
fn apply_composer_mirror(source: &str) -> Result<ApplyResult> {
    let mirror_url =
        config::resolve_mirror_url("composer", source).unwrap_or_else(|_| source.to_string());

    println!("    {} Composer 镜像请手动执行:", "⚠".yellow());
    println!("      composer config -g repo.packagist composer {mirror_url}");

    Ok(ApplyResult::ManualRequired(
        "Composer 镜像请手动执行 composer config 命令".into(),
    ))
}

/// 应用 yarn 镜像
fn apply_yarn_mirror(url: &str) -> Result<ApplyResult> {
    let yarnrc = paths::yarnrc_path()?;

    let new_line = format!("registry \"{url}\"");

    if yarnrc.exists() {
        // 保留其他配置，只替换 registry 行
        let content = std::fs::read_to_string(&yarnrc)?;
        let mut found = false;
        let new_content: String = content
            .lines()
            .map(|line| {
                if line.trim().starts_with("registry ") {
                    found = true;
                    new_line.clone()
                } else {
                    line.to_string()
                }
            })
            .collect::<Vec<_>>()
            .join("\n");
        if found {
            std::fs::write(&yarnrc, new_content)?;
        } else {
            // 没有 registry 行，追加
            let mut c = new_content;
            if !c.ends_with('\n') {
                c.push('\n');
            }
            c.push_str(&format!("{new_line}\n"));
            std::fs::write(&yarnrc, c)?;
        }
    } else {
        std::fs::write(&yarnrc, format!("{new_line}\n"))?;
    }

    info!("wrote yarn config to {}", yarnrc.display());
    Ok(ApplyResult::Applied)
}

/// 应用 pnpm 镜像
fn apply_pnpm_mirror(url: &str) -> Result<ApplyResult> {
    // pnpm 使用 .npmrc
    apply_npm_mirror(url)
}

/// 应用 Gradle 镜像 (写 init.gradle，使用 Maven 仓库镜像)
fn apply_gradle_mirror(source: &str) -> Result<ApplyResult> {
    // init.gradle 配置的是 Maven 仓库镜像（用于下载依赖）
    let maven_url =
        config::resolve_mirror_url("gradle", source).unwrap_or_else(|_| source.to_string());

    let init_path = paths::gradle_init_path()?;

    if let Some(gradle_home) = init_path.parent() {
        std::fs::create_dir_all(gradle_home)?;
    }

    // 如果 init.gradle 已有其他内容，备份后重写
    if init_path.exists() {
        let existing = std::fs::read_to_string(&init_path)?;
        if !existing.is_empty() && !existing.contains("nz-switch") {
            let backup = init_path.with_extension("gradle.bak");
            std::fs::write(&backup, &existing)?;
            info!("backed up existing init.gradle to {}", backup.display());
        }
    }

    let content = format!(
        "// nz-switch mirror config\nallprojects {{\n  repositories {{\n    maven {{ url '{maven_url}' }}\n    mavenCentral()\n  }}\n}}\n"
    );

    std::fs::write(&init_path, content)?;
    info!("wrote gradle init script to {}", init_path.display());

    Ok(ApplyResult::Applied)
}

/// 应用 CocoaPods 镜像 (仅打印提示)
fn apply_cocoapods_mirror(source: &str) -> Result<ApplyResult> {
    let mirror_url =
        config::resolve_mirror_url("cocoapods", source).unwrap_or_else(|_| source.to_string());

    println!("    {} CocoaPods 镜像请手动执行:", "⚠".yellow());
    println!("      pod repo remove master");
    println!("      pod repo add master {mirror_url}");

    Ok(ApplyResult::ManualRequired(
        "CocoaPods 镜像请手动执行 pod repo 命令".into(),
    ))
}

/// 应用 VS Code 扩展市场镜像 (仅打印提示)
fn apply_vscode_mirror(source: &str) -> Result<ApplyResult> {
    let mirror_url =
        config::resolve_mirror_url("vscode", source).unwrap_or_else(|_| source.to_string());

    println!(
        "    {} VS Code 扩展市场镜像请在 settings.json 中配置:",
        "⚠".yellow()
    );
    println!("      \"extensions.gallery\": {{");
    println!("        \"serviceUrl\": \"{mirror_url}\"");
    println!("      }}");

    Ok(ApplyResult::ManualRequired(
        "VS Code 扩展市场镜像请在 settings.json 中配置".into(),
    ))
}

/// 应用 Android Google Maven 镜像 (仅打印提示)
fn apply_android_maven_mirror(source: &str) -> Result<ApplyResult> {
    let (google_url, gradle_url) = match source {
        "aliyun-google" | "aliyun-gradle" => (
            "https://maven.aliyun.com/repository/google",
            "https://maven.aliyun.com/repository/gradle-plugin",
        ),
        _ => {
            println!(
                "    {} 暂不支持 Android Maven 镜像源: {}",
                "⚠".yellow(),
                source
            );
            return Ok(ApplyResult::Applied);
        }
    };

    println!(
        "    {} Android Maven 镜像请在 build.gradle 中配置:",
        "⚠".yellow()
    );
    println!("      repositories {{");
    println!("        maven {{ url '{google_url}' }}");
    println!("        maven {{ url '{gradle_url}' }}");
    println!("      }}");

    Ok(ApplyResult::ManualRequired(
        "Android Maven 镜像请在 build.gradle 中配置".into(),
    ))
}

/// 应用 Android Gradle Wrapper 镜像 (仅打印提示)
fn apply_android_gradle_mirror(source: &str) -> Result<ApplyResult> {
    let mirror_url =
        config::resolve_mirror_url("android-gradle", source).unwrap_or_else(|_| source.to_string());

    println!(
        "    {} Gradle Wrapper 镜像请在 gradle-wrapper.properties 中配置:",
        "⚠".yellow()
    );
    println!("      distributionUrl={mirror_url}gradle-X-all.zip");

    Ok(ApplyResult::ManualRequired(
        "Gradle Wrapper 镜像请在 gradle-wrapper.properties 中配置".into(),
    ))
}

/// 应用 Swift Package Manager 镜像 (仅打印提示)
fn apply_swift_mirror(source: &str) -> Result<ApplyResult> {
    let mirror_url =
        config::resolve_mirror_url("swift", source).unwrap_or_else(|_| source.to_string());

    println!("    {} Swift Package Manager 镜像:", "⚠".yellow());
    println!("      请使用代理访问 github.com 依赖，或使用国内 Git 服务镜像");
    println!("      参考: {mirror_url}");

    Ok(ApplyResult::ManualRequired(format!(
        "Swift 镜像需要手动配置，请参考: {mirror_url}"
    )))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_mirror_url_preset() {
        // "tuna" 应解析为实际 URL
        let url = config::resolve_mirror_url("pip", "tuna").unwrap();
        assert!(url.starts_with("http"), "expected URL, got: {}", url);
        assert!(!url.is_empty());
    }

    #[test]
    fn test_resolve_mirror_url_raw() {
        // 已经是 URL 的应直接返回
        let raw = "https://custom-mirror.example.com/simple";
        let url = config::resolve_mirror_url("pip", raw).unwrap();
        assert_eq!(url, raw);
    }

    #[test]
    fn test_resolve_mirror_url_unknown() {
        // 未知预设名应返回错误
        let result = config::resolve_mirror_url("pip", "nonexistent-preset-xyz");
        assert!(
            result.is_err(),
            "expected error for unknown preset, got: {:?}",
            result
        );
    }

    #[test]
    fn test_pip_config_path_not_empty() {
        let path = paths::pip_config_path().unwrap();
        assert!(!path.to_string_lossy().is_empty());
    }

    #[test]
    fn test_cargo_config_path() {
        let path = paths::cargo_config_path().unwrap();
        assert!(path.to_string_lossy().contains(".cargo"));
        assert!(path.to_string_lossy().contains("config.toml"));
    }
}
