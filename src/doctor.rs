use anyhow::Result;
use colored::Colorize;
use serde::Serialize;
use crate::format;
use crate::mirror;

/// 单项诊断结果
#[derive(Serialize, Clone)]
pub struct DoctorCheck {
    pub name: String,
    pub status: String, // "ok", "warn", "error"
    pub message: String,
}

/// 运行所有诊断检查（结构化返回，CLI 和 Tauri 共用）
/// `domestic` 为 true 时表示国内环境，未配置镜像/代理会报 warn；
/// 为 false 时表示国际环境，使用默认源是正常状态。
pub fn run_checks(domestic: bool) -> Vec<DoctorCheck> {
    vec![
        check_config(),
        check_pip(domestic),
        check_npm(domestic),
        check_cargo(domestic),
        check_go(domestic),
        check_git(),
        check_proxy_env(),
        check_conda(domestic),
        check_brew(domestic),
        check_yarn(domestic),
        check_docker(domestic),
        check_dns(domestic),
        check_local_config(),
    ]
}

/// CLI 用：运行环境诊断并打印
pub fn run_diagnosis() -> Result<()> {
    println!("{}", "🩺 环境诊断".cyan().bold());
    println!();

    // 读取当前 profile 判断是否国内环境
    let domestic = crate::config::AppConfig::load()
        .map(|cfg| {
            cfg.profiles.get(&cfg.current_profile)
                .is_some_and(is_domestic_profile)
        })
        .unwrap_or(false);

    let checks = run_checks(domestic);
    let mut warnings = 0;
    let mut errors = 0;

    let mut table = format::new_table(&["状态", "检查项", "详情"]);

    for check in &checks {
        let icon = match check.status.as_str() {
            "ok" => format::ok(),
            "warn" => {
                warnings += 1;
                format::warn()
            }
            _ => {
                errors += 1;
                format::err()
            }
        };
        table.add_row(vec![icon, check.name.bold().to_string(), check.message.clone()]);
    }

    println!("{table}");
    println!();

    if errors == 0 && warnings == 0 {
        println!("{} 一切正常！", "🎉".green());
    } else {
        println!(
            "诊断完成: {} 个警告, {} 个错误",
            warnings.to_string().yellow(),
            errors.to_string().red()
        );
        if !domestic {
            println!("运行 {} 切换到国内配置。", "nz-switch switch cn".cyan());
        } else {
            println!("运行 {} 来自动配置。", "nz-switch switch cn".cyan());
        }
    }

    Ok(())
}

/// 已知的国内镜像域名关键词
const CN_MIRROR_KEYWORDS: &[&str] = &[
    "tuna", "ustc", "aliyun", "npmmirror", "taobao",
    "rsproxy", "goproxy.cn", "goproxy.io", "hf-mirror",
    "huaweicloud", "tencent", "163.com",
];

/// 判断 URL 是否为国内镜像
fn is_cn_mirror_url(url: &str) -> bool {
    CN_MIRROR_KEYWORDS.iter().any(|kw| url.contains(kw))
}

/// 判断是否为国内 profile（有中国镜像配置）
pub fn is_domestic_profile(profile: &crate::profile::Profile) -> bool {
    let has_cn_mirror = profile.mirrors.values().any(|v| is_cn_mirror_url(v));
    let has_cn_dns = profile.dns.as_ref().is_some_and(|d| {
        d.servers.iter().any(|s| crate::dns::is_domestic_dns(s))
    });
    has_cn_mirror || has_cn_dns
}

/// 通用镜像检测：使用 detect_current_mirror 统一检测，消除各工具的重复文件读取逻辑
fn check_mirror(tool: &str, display_name: &str, domestic: bool) -> DoctorCheck {
    let detected = mirror::detect_current_mirror(tool);

    match detected {
        Some(name) => {
            // 获取镜像 URL 以判断是否为国内镜像
            let url = mirror::config::find_mirror_def(tool)
                .and_then(|def| def.mirrors.iter().find(|m| m.name == name))
                .map(|m| m.url.clone());

            let is_default = url.as_deref().is_some_and(|u| {
                mirror::config::find_mirror_def(tool)
                    .and_then(|def| def.mirrors.iter().find(|m| m.display_name == "官方"))
                    .is_some_and(|official| official.url.trim_end_matches('/') == u.trim_end_matches('/'))
            });

            if is_default {
                DoctorCheck {
                    name: display_name.to_string(),
                    status: if domestic { "warn" } else { "ok" }.to_string(),
                    message: if domestic { format!("{name} 使用默认源") } else { "使用默认源".to_string() },
                }
            } else if url.as_deref().is_some_and(|u| is_cn_mirror_url(u)) || is_cn_mirror_url(&name) {
                DoctorCheck {
                    name: display_name.to_string(),
                    status: "ok".to_string(),
                    message: format!("已配置国内镜像: {name}"),
                }
            } else {
                DoctorCheck {
                    name: display_name.to_string(),
                    status: if domestic { "warn" } else { "ok" }.to_string(),
                    message: format!("使用非国内镜像: {name}"),
                }
            }
        }
        None => DoctorCheck {
            name: display_name.to_string(),
            status: if domestic { "warn" } else { "ok" }.to_string(),
            message: if domestic { format!("未配置 {display_name}") } else { "使用默认源".to_string() },
        },
    }
}

fn check_config() -> DoctorCheck {
    match crate::config::config_path() {
        Ok(path) => {
            if path.exists() {
                DoctorCheck {
                    name: "配置文件".to_string(),
                    status: "ok".to_string(),
                    message: format!("存在: {}", path.display()),
                }
            } else {
                DoctorCheck {
                    name: "配置文件".to_string(),
                    status: "warn".to_string(),
                    message: "不存在，运行 nz-switch init 初始化".to_string(),
                }
            }
        }
        Err(e) => DoctorCheck {
            name: "配置文件".to_string(),
            status: "error".to_string(),
            message: format!("获取路径失败: {e}"),
        },
    }
}

fn check_pip(domestic: bool) -> DoctorCheck {
    check_mirror("pip", "pip 镜像", domestic)
}

fn check_npm(domestic: bool) -> DoctorCheck {
    check_mirror("npm", "npm 镜像", domestic)
}

fn check_cargo(domestic: bool) -> DoctorCheck {
    check_mirror("cargo", "cargo 镜像", domestic)
}

fn check_go(domestic: bool) -> DoctorCheck {
    match std::env::var("GOPROXY") {
        Ok(val) => {
            if is_cn_mirror_url(&val) {
                DoctorCheck {
                    name: "Go 代理".to_string(),
                    status: "ok".to_string(),
                    message: format!("GOPROXY={val}"),
                }
            } else if domestic {
                DoctorCheck {
                    name: "Go 代理".to_string(),
                    status: "warn".to_string(),
                    message: format!("使用非国内代理: {val}"),
                }
            } else {
                DoctorCheck {
                    name: "Go 代理".to_string(),
                    status: "ok".to_string(),
                    message: format!("GOPROXY={val}"),
                }
            }
        }
        Err(_) => DoctorCheck {
            name: "Go 代理".to_string(),
            status: if domestic { "warn" } else { "ok" }.to_string(),
            message: if domestic { "GOPROXY 环境变量未设置" } else { "使用默认代理" }.to_string(),
        },
    }
}

fn check_git() -> DoctorCheck {
    let output = std::process::Command::new("git")
        .arg("config")
        .arg("--global")
        .arg("http.proxy")
        .output();

    match output {
        Ok(o) if o.status.success() => {
            let proxy = String::from_utf8_lossy(&o.stdout).trim().to_string();
            DoctorCheck {
                name: "Git 配置".to_string(),
                status: "ok".to_string(),
                message: format!("Git 代理: {proxy}"),
            }
        }
        _ => DoctorCheck {
            name: "Git 配置".to_string(),
            status: "ok".to_string(),
            message: "未配置 Git 代理 (可选)".to_string(),
        },
    }
}

fn check_proxy_env() -> DoctorCheck {
    let http_proxy = std::env::var("HTTP_PROXY").or_else(|_| std::env::var("http_proxy"));
    let https_proxy = std::env::var("HTTPS_PROXY").or_else(|_| std::env::var("https_proxy"));

    match (http_proxy, https_proxy) {
        (Ok(http), Ok(https)) => DoctorCheck {
            name: "代理环境变量".to_string(),
            status: "ok".to_string(),
            message: format!("HTTP={http}, HTTPS={https}"),
        },
        (Ok(http), Err(_)) => DoctorCheck {
            name: "代理环境变量".to_string(),
            status: "warn".to_string(),
            message: format!("仅设置了 HTTP_PROXY={http}"),
        },
        (Err(_), Ok(https)) => DoctorCheck {
            name: "代理环境变量".to_string(),
            status: "warn".to_string(),
            message: format!("仅设置了 HTTPS_PROXY={https}"),
        },
        (Err(_), Err(_)) => DoctorCheck {
            name: "代理环境变量".to_string(),
            status: "ok".to_string(),
            message: "未设置代理环境变量 (可选)".to_string(),
        },
    }
}

fn check_conda(domestic: bool) -> DoctorCheck {
    check_mirror("conda", "Conda 镜像", domestic)
}

fn check_brew(domestic: bool) -> DoctorCheck {
    // Homebrew 仅在 macOS/Linux 上适用
    if cfg!(target_os = "windows") {
        return DoctorCheck {
            name: "Homebrew".to_string(),
            status: "ok".to_string(),
            message: "Windows 平台不适用".to_string(),
        };
    }

    match std::env::var("HOMEBREW_BREW_GIT_REMOTE") {
        Ok(val) => {
            if is_cn_mirror_url(&val) {
                DoctorCheck {
                    name: "Homebrew 镜像".to_string(),
                    status: "ok".to_string(),
                    message: format!("已配置国内镜像: {val}"),
                }
            } else {
                DoctorCheck {
                    name: "Homebrew 镜像".to_string(),
                    status: if domestic { "warn" } else { "ok" }.to_string(),
                    message: format!("使用非国内镜像: {val}"),
                }
            }
        }
        Err(_) => {
            // 检查 brew 是否安装
            if which::which("brew").is_ok() {
                DoctorCheck {
                    name: "Homebrew 镜像".to_string(),
                    status: if domestic { "warn" } else { "ok" }.to_string(),
                    message: if domestic { "brew 已安装但未配置镜像源" } else { "brew 已安装，使用默认源" }.to_string(),
                }
            } else {
                DoctorCheck {
                    name: "Homebrew 镜像".to_string(),
                    status: "ok".to_string(),
                    message: "brew 未安装".to_string(),
                }
            }
        }
    }
}

fn check_yarn(domestic: bool) -> DoctorCheck {
    check_mirror("yarn", "yarn 镜像", domestic)
}

fn check_docker(domestic: bool) -> DoctorCheck {
    check_mirror("docker", "Docker 镜像", domestic)
}

fn check_dns(domestic: bool) -> DoctorCheck {
    // 读取 /etc/resolv.conf 检查 DNS 配置
    let resolv_path = std::path::Path::new("/etc/resolv.conf");
    if !resolv_path.exists() {
        return DoctorCheck {
            name: "DNS 配置".to_string(),
            status: "ok".to_string(),
            message: "非 Linux 平台或 resolv.conf 不存在".to_string(),
        };
    }

    let content = std::fs::read_to_string(resolv_path).unwrap_or_default();
    let servers: Vec<&str> = content.lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            if trimmed.starts_with("nameserver") {
                trimmed.split_whitespace().nth(1)
            } else {
                None
            }
        })
        .collect();

    if servers.is_empty() {
        DoctorCheck {
            name: "DNS 配置".to_string(),
            status: "warn".to_string(),
            message: "未检测到 DNS 服务器配置".to_string(),
        }
    } else {
        // 检查是否使用国内常见 DNS（基于 dns.rs 中的 DNS_PRESETS）
        let has_domestic = servers.iter().any(|s| crate::dns::is_domestic_dns(s));
        let has_foreign = servers.iter().any(|s| crate::dns::is_foreign_dns(s));

        if has_domestic {
            DoctorCheck {
                name: "DNS 配置".to_string(),
                status: if domestic { "ok" } else { "warn" }.to_string(),
                message: format!("使用国内 DNS: {}", servers.join(", ")),
            }
        } else if has_foreign {
            DoctorCheck {
                name: "DNS 配置".to_string(),
                status: if domestic { "warn" } else { "ok" }.to_string(),
                message: format!("使用海外 DNS: {}", servers.join(", ")),
            }
        } else {
            DoctorCheck {
                name: "DNS 配置".to_string(),
                status: "ok".to_string(),
                message: format!("DNS: {}", servers.join(", ")),
            }
        }
    }
}

fn check_local_config() -> DoctorCheck {
    match crate::local_config::find_local_config() {
        Some(path) => DoctorCheck {
            name: "项目配置".to_string(),
            status: "ok".to_string(),
            message: format!("检测到: {}", path.display()),
        },
        None => DoctorCheck {
            name: "项目配置".to_string(),
            status: "ok".to_string(),
            message: "无项目级配置 (.nz-switch.toml)".to_string(),
        },
    }
}
