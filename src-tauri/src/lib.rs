use serde::Serialize;
use tauri::Emitter;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// 操作互斥锁：switch_profile 和 test_mirrors_streaming 互斥执行，
/// 避免 unsafe env::set_var 与多线程测速并发导致数据竞争
struct OpMutex(Arc<Mutex<()>>);

// ─── Tauri Commands ──────────────────────────────────────────────────

/// 获取当前状态
#[tauri::command]
fn get_status() -> Result<StatusInfo, String> {
    let cfg = nz_switch::config::AppConfig::load().map_err(|e| e.to_string())?;
    let profile = nz_switch::profile::resolve_profile(&cfg.current_profile).map_err(|e| e.to_string())?;

    let local = nz_switch::local_config::load_local_config().map_err(|e| e.to_string())?;
    let effective = match &local {
        Some(lc) => nz_switch::local_config::merge_with_local(&profile, lc, &cfg).map_err(|e| e.to_string())?,
        None => profile.clone(),
    };

    Ok(StatusInfo {
        current_profile: cfg.current_profile.clone(),
        display_name: profile.display_name.clone(),
        env: effective.env.clone(),
        mirrors: effective.mirrors.clone(),
        proxy: effective.proxy.clone(),
        git: effective.git.clone(),
        dns: effective.dns.clone(),
        has_local_config: local.is_some(),
    })
}

/// 切换 profile
#[tauri::command]
fn switch_profile(state: tauri::State<'_, OpMutex>, name: String) -> Result<String, String> {
    let _guard = state.0.lock().map_err(|e| e.to_string())?;
    // guard 在函数返回时 drop，释放锁
    let profile = nz_switch::profile::resolve_profile(&name).map_err(|e| e.to_string())?;
    let result = nz_switch::switch_profile(&name).map_err(|e| e.to_string())?;

    let mut msg = format!("已切换到 {} 环境", profile.display_name);
    if !result.errors.is_empty() {
        msg.push_str(&format!(" (部分失败: {})", result.errors.join("; ")));
    }
    if !result.manual_instructions.is_empty() {
        msg.push_str(&format!(" ({} 项需手动配置)", result.manual_instructions.len()));
    }
    Ok(msg)
}

/// 获取配置
#[tauri::command]
fn get_config() -> Result<ConfigInfo, String> {
    let cfg = nz_switch::config::AppConfig::load().map_err(|e| e.to_string())?;

    let mut profiles = HashMap::new();
    for (name, profile) in &cfg.profiles {
        profiles.insert(name.clone(), ProfileInfo {
            display_name: profile.display_name.clone(),
            env: profile.env.clone(),
            mirrors: profile.mirrors.clone(),
            proxy: profile.proxy.clone(),
            git: profile.git.clone(),
            dns: profile.dns.clone(),
        });
    }

    Ok(ConfigInfo {
        current_profile: cfg.current_profile.clone(),
        profiles,
        config_path: nz_switch::config::config_path()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default(),
    })
}

/// 列出镜像源（按平台过滤）
#[tauri::command]
fn list_mirrors(tool: Option<String>) -> Result<Vec<MirrorGroup>, String> {
    let defs = nz_switch::mirror::config::load_mirror_defs();
    let platforms = nz_switch::mirror::config::load_platform_mirrors();

    let mut groups = Vec::new();
    for (tool_name, mirrors) in &platforms {
        if let Some(ref t) = tool {
            if t != tool_name {
                continue;
            }
        }
        // 查找对应的镜像源定义以获取 display_name
        let mirror_def = defs.iter().find(|d| &d.tool == tool_name);
        let items: Vec<MirrorItem> = mirrors.iter()
            .map(|(name, url)| {
                let display_name = mirror_def
                    .and_then(|def| def.mirrors.iter().find(|m| &m.name == name))
                    .map(|m| m.display_name.clone())
                    .unwrap_or_else(|| name.clone());
                MirrorItem {
                    name: name.clone(),
                    display_name,
                    url: url.clone(),
                }
            })
            .collect();
        let (display_name, config_type, config_path) = mirror_def
            .map(|d| (d.display_name.clone(), d.config_type.clone(), d.config_path.clone()))
            .unwrap_or_else(|| (tool_name.clone(), "unknown".into(), None));
        groups.push(MirrorGroup {
            tool: tool_name.clone(),
            display_name,
            config_type,
            config_path,
            mirrors: items,
        });
    }

    Ok(groups)
}

/// 获取平台信息
#[tauri::command]
fn get_platform_info() -> PlatformInfo {
    let (id, name) = nz_switch::platform_info();
    PlatformInfo { id, name }
}

/// 获取已安装的工具列表
#[tauri::command]
fn get_installed_tools() -> Vec<String> {
    nz_switch::mirror::config::installed_tools()
}

/// 检测所有已安装工具的当前镜像源（读取系统实际配置文件/环境变量）
#[tauri::command]
fn detect_mirrors() -> HashMap<String, String> {
    nz_switch::mirror::detect_all_mirrors()
}

/// 设置镜像源
#[tauri::command]
fn set_mirror(state: tauri::State<'_, OpMutex>, tool: String, source: String) -> Result<String, String> {
    let _guard = state.0.lock().map_err(|e| e.to_string())?;
    nz_switch::mirror::set_mirror(&tool, &source).map_err(|e| e.to_string())?;
    Ok(format!("{tool} 镜像源已设置为: {source}"))
}

/// 重置镜像源
#[tauri::command]
fn reset_mirror(state: tauri::State<'_, OpMutex>, tool: String) -> Result<String, String> {
    let _guard = state.0.lock().map_err(|e| e.to_string())?;
    nz_switch::mirror::reset_mirror(&tool).map_err(|e| e.to_string())?;
    Ok(format!("{tool} 镜像源已重置"))
}

/// 获取 DNS 预设
#[tauri::command]
fn get_dns_presets() -> Vec<DnsPreset> {
    nz_switch::dns::DNS_PRESETS.iter()
        .map(|(name, servers)| DnsPreset {
            name: name.to_string(),
            servers: servers.iter().map(|s| s.to_string()).collect(),
        })
        .collect()
}

/// 显示当前 DNS
#[tauri::command]
fn get_current_dns() -> Vec<String> {
    #[cfg(target_os = "macos")]
    {
        // macOS: 通过 scutil 获取 DNS
        if let Ok(output) = std::process::Command::new("scutil")
            .args(["--dns"])
            .output()
        {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let mut servers = Vec::new();
            for line in stdout.lines() {
                if let Some(rest) = line.trim().strip_prefix("nameserver[") {
                    // "nameserver[0] : 8.8.8.8"
                    if let Some(ip) = rest.split(':').nth(1) {
                        let ip = ip.trim().to_string();
                        if !servers.contains(&ip) {
                            servers.push(ip);
                        }
                    }
                }
            }
            return servers;
        }
    }

    #[cfg(target_os = "windows")]
    {
        // Windows: 通过 PowerShell 获取 DNS
        if let Ok(output) = std::process::Command::new("powershell")
            .args(["-Command", "(Get-DnsClientServerAddress).ServerAddresses"])
            .output()
        {
            let stdout = String::from_utf8_lossy(&output.stdout);
            return stdout.lines()
                .map(|l| l.trim().to_string())
                .filter(|l| !l.is_empty())
                .collect();
        }
    }

    // Linux / fallback: /etc/resolv.conf
    let content = std::fs::read_to_string("/etc/resolv.conf").unwrap_or_default();
    content.lines()
        .filter(|line| line.trim().starts_with("nameserver"))
        .filter_map(|line| line.split_whitespace().nth(1).map(|s| s.to_string()))
        .collect()
}

/// 运行诊断（复用库函数）
#[tauri::command]
fn run_doctor() -> Vec<nz_switch::doctor::DoctorCheck> {
    let domestic = nz_switch::config::AppConfig::load()
        .map(|cfg| {
            cfg.profiles.get(&cfg.current_profile)
                .is_some_and(nz_switch::doctor::is_domestic_profile)
        })
        .unwrap_or(false);
    nz_switch::doctor::run_checks(domestic)
}

/// 重置配置
#[tauri::command]
fn reset_config(state: tauri::State<'_, OpMutex>) -> Result<String, String> {
    let _guard = state.0.lock().map_err(|e| e.to_string())?;
    let config_path = nz_switch::config::config_path().map_err(|e| e.to_string())?;
    let cfg = nz_switch::config::AppConfig::default();
    cfg.save(&config_path).map_err(|e| e.to_string())?;
    Ok("配置已重置为默认值".to_string())
}

/// 获取原始配置 (TOML)
#[tauri::command]
fn get_raw_config() -> Result<RawConfigInfo, String> {
    let config_path = nz_switch::config::config_path().map_err(|e| e.to_string())?;
    let path_str = config_path.to_string_lossy().to_string();

    if !config_path.exists() {
        let cfg = nz_switch::config::AppConfig::default();
        let toml = toml::to_string_pretty(&cfg).map_err(|e| e.to_string())?;
        return Ok(RawConfigInfo { toml, path: path_str });
    }

    let content = std::fs::read_to_string(&config_path).map_err(|e| e.to_string())?;
    // 验证 TOML 合法性
    let _cfg: nz_switch::config::AppConfig = toml::from_str(&content)
        .map_err(|e| format!("TOML 解析失败: {e}"))?;
    Ok(RawConfigInfo { toml: content, path: path_str })
}

/// 保存原始配置 (TOML)
#[tauri::command]
fn save_raw_config(state: tauri::State<'_, OpMutex>, toml: String) -> Result<String, String> {
    let _guard = state.0.lock().map_err(|e| e.to_string())?;
    let cfg: nz_switch::config::AppConfig = toml::from_str(&toml)
        .map_err(|e| format!("TOML 解析失败: {e}"))?;

    let config_path = nz_switch::config::config_path().map_err(|e| e.to_string())?;
    cfg.save(&config_path).map_err(|e| e.to_string())?;

    Ok("配置已保存".to_string())
}

/// 并发测速镜像源（批量返回，保留兼容）
#[tauri::command]
fn test_mirrors(tool: Option<String>) -> Result<Vec<nz_switch::mirror::MirrorTestResult>, String> {
    let results = nz_switch::mirror::test_mirrors_concurrent(tool.as_deref());
    Ok(results)
}

/// 流式测速：每完成一个镜像就通过事件通知前端
/// 仅执行 TCP 延迟探测，不修改环境变量，无需持锁
#[tauri::command]
fn test_mirrors_streaming(app: tauri::AppHandle, tool: Option<String>) {
    std::thread::spawn(move || {
        let app_for_cb = app.clone();
        nz_switch::mirror::test_mirrors_streaming(tool.as_deref(), move |result| {
            let _ = app_for_cb.emit("mirror-test-result", result);
        });
        let _ = app.emit("mirror-test-done", ());
    });
}

// ─── 数据结构 ────────────────────────────────────────────────────────

/// 镜像分组（用于前端列表展示）
#[derive(Serialize)]
struct MirrorGroup {
    tool: String,
    display_name: String,
    config_type: String,
    config_path: Option<String>,
    mirrors: Vec<MirrorItem>,
}

/// 单个镜像条目
#[derive(Serialize)]
struct MirrorItem {
    name: String,
    display_name: String,
    url: String,
}

/// DNS 预设
#[derive(Serialize)]
struct DnsPreset {
    name: String,
    servers: Vec<String>,
}

/// 原始配置信息
#[derive(Serialize)]
struct RawConfigInfo {
    toml: String,
    path: String,
}

/// 平台信息
#[derive(Serialize)]
struct PlatformInfo {
    id: String,
    name: String,
}

/// IP 归属地信息
#[derive(Serialize)]
struct IpLocation {
    ip: String,
    country: String,
    region: String,
    city: String,
    is_cn: bool,
}

/// 状态信息（组合结构，用于 get_status）
#[derive(Serialize)]
struct StatusInfo {
    current_profile: String,
    display_name: String,
    env: HashMap<String, String>,
    mirrors: HashMap<String, String>,
    proxy: Option<nz_switch::proxy::ProxyConfig>,
    git: Option<nz_switch::git::GitConfig>,
    dns: Option<nz_switch::dns::DnsConfig>,
    has_local_config: bool,
}

/// 配置信息（用于 get_config）
#[derive(Serialize)]
struct ConfigInfo {
    current_profile: String,
    profiles: HashMap<String, ProfileInfo>,
    config_path: String,
}

/// Profile 信息（用于 get_config 返回各 profile）
#[derive(Serialize)]
struct ProfileInfo {
    display_name: String,
    env: HashMap<String, String>,
    mirrors: HashMap<String, String>,
    proxy: Option<nz_switch::proxy::ProxyConfig>,
    git: Option<nz_switch::git::GitConfig>,
    dns: Option<nz_switch::dns::DnsConfig>,
}

/// 获取当前 IP 归属地（通过 ip-api.com）
#[tauri::command]
async fn get_ip_location() -> Result<IpLocation, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .map_err(|e| e.to_string())?;

    let resp = client
        .get("http://ip-api.com/json/?lang=zh-CN&fields=query,country,regionName,city,countryCode")
        .send()
        .await
        .map_err(|e| format!("请求 IP 归属地失败: {e}"))?;

    let json: serde_json::Value = resp.json().await.map_err(|e| format!("解析 IP 归属地失败: {e}"))?;

    let ip = json["query"].as_str().unwrap_or("unknown").to_string();
    let country = json["country"].as_str().unwrap_or("未知").to_string();
    let region = json["regionName"].as_str().unwrap_or("").to_string();
    let city = json["city"].as_str().unwrap_or("").to_string();
    let country_code = json["countryCode"].as_str().unwrap_or("");

    let is_cn = country_code == "CN";

    Ok(IpLocation { ip, country, region, city, is_cn })
}

// ─── 入口 ────────────────────────────────────────────────────────────

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // AppImage 内运行时，设置环境变量避免 GPU 驱动冲突
    // 打包的 libwayland/libEGL/libGL 与宿主机驱动版本不匹配会导致 EGL_BAD_ALLOC
    if std::env::var("APPIMAGE").is_ok() {
        // 回退到 X11，避免 Wayland EGL surface 创建失败
        std::env::set_var("GDK_BACKEND", "x11");
        // 禁用 WebKit DMA-BUF 渲染器，避免 EGL 分配失败
        std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
        // 禁用 WebKit 合成模式，避免 GPU 加速问题
        std::env::set_var("WEBKIT_DISABLE_COMPOSITING_MODE", "1");
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::default().build())
        .manage(OpMutex(Arc::new(Mutex::new(()))))
        .invoke_handler(tauri::generate_handler![
            get_status,
            switch_profile,
            get_config,
            list_mirrors,
            set_mirror,
            reset_mirror,
            get_dns_presets,
            get_current_dns,
            run_doctor,
            reset_config,
            get_raw_config,
            save_raw_config,
            test_mirrors,
            test_mirrors_streaming,
            get_installed_tools,
            detect_mirrors,
            get_platform_info,
            get_ip_location,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
