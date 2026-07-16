use serde::Deserialize;
use std::collections::HashMap;
use std::sync::OnceLock;

// ─── 配置数据结构 ──────────────────────────────────────────────────

/// 单个镜像源条目
#[derive(Deserialize, Clone, Debug)]
pub struct MirrorEntry {
    pub name: String,
    pub display_name: String,
    pub url: String,
    pub os: Vec<String>,   // 空 = 所有平台
    pub arch: Vec<String>, // 空 = 所有架构
    pub enabled: bool,
    /// 可选的环境变量映射 (env_var_name -> value)
    /// 用于 brew/pub 等需要多个不同环境变量的工具
    #[serde(default)]
    pub env_vars: Option<HashMap<String, String>>,
}

/// 工具的镜像源定义文件
#[derive(Deserialize, Clone, Debug)]
pub struct MirrorDef {
    pub tool: String,
    pub display_name: String,
    pub description: String,
    pub config_type: String, // "file", "env", "manual"
    pub config_path: Option<String>,
    pub os: Vec<String>,   // 空 = 所有平台
    pub arch: Vec<String>, // 空 = 所有架构
    pub mirrors: Vec<MirrorEntry>,
}

/// 平台配置中的工具条目
#[derive(Deserialize, Clone, Debug)]
pub struct PlatformTool {
    pub tool: String,
    pub priority: u32,
}

/// 平台配置文件
#[derive(Deserialize, Clone, Debug)]
pub struct PlatformDef {
    pub platform: String,
    pub display_name: String,
    pub distro: Option<String>, // 可选: archlinux, ubuntu, centos 等
    pub os: Option<String>,     // 可选: linux, macos, windows
    pub arch: Option<String>,   // 可选: x86_64, aarch64
    pub tools: Vec<PlatformTool>,
}

// ─── 内嵌默认配置 ──────────────────────────────────────────────────

macro_rules! include_mirror {
    ($name:expr) => {
        include_str!(concat!("../../config/mirrors/", $name, ".json"))
    };
}

macro_rules! include_platform {
    ($path:expr) => {
        include_str!(concat!("../../config/platforms/", $path))
    };
}

const MIRROR_PIP: &str = include_mirror!("pip");
const MIRROR_NPM: &str = include_mirror!("npm");
const MIRROR_YARN: &str = include_mirror!("yarn");
const MIRROR_PNPM: &str = include_mirror!("pnpm");
const MIRROR_BUN: &str = include_mirror!("bun");
const MIRROR_DENO: &str = include_mirror!("deno");
const MIRROR_CARGO: &str = include_mirror!("cargo");
const MIRROR_GO: &str = include_mirror!("go");
const MIRROR_DOCKER: &str = include_mirror!("docker");
const MIRROR_K8S_GCR: &str = include_mirror!("k8s-gcr");
const MIRROR_K8S_REGISTRY: &str = include_mirror!("k8s-registry");
const MIRROR_GHCR: &str = include_mirror!("ghcr");
const MIRROR_QUAY: &str = include_mirror!("quay");
const MIRROR_CONDA: &str = include_mirror!("conda");
const MIRROR_BREW: &str = include_mirror!("brew");
const MIRROR_APT: &str = include_mirror!("apt");
const MIRROR_CHOCO: &str = include_mirror!("choco");
const MIRROR_NUGET: &str = include_mirror!("nuget");
const MIRROR_MAVEN: &str = include_mirror!("maven");
const MIRROR_GRADLE: &str = include_mirror!("gradle");
const MIRROR_RUBYGEMS: &str = include_mirror!("rubygems");
const MIRROR_COMPOSER: &str = include_mirror!("composer");
const MIRROR_PUB: &str = include_mirror!("pub");
const MIRROR_COCOAPODS: &str = include_mirror!("cocoapods");
const MIRROR_HUGGINGFACE: &str = include_mirror!("huggingface");
const MIRROR_NODEJS: &str = include_mirror!("nodejs");
const MIRROR_PYTHON: &str = include_mirror!("python");
const MIRROR_RUSTUP: &str = include_mirror!("rustup");
const MIRROR_VSCODE: &str = include_mirror!("vscode");
const MIRROR_ANDROID_MAVEN: &str = include_mirror!("android-maven");
const MIRROR_ANDROID_GRADLE: &str = include_mirror!("android-gradle");
const MIRROR_SWIFT: &str = include_mirror!("swift");

// 平台配置: os.arch.json, 可选 distro: os.distro.arch.json
const PLATFORM_LINUX_X86: &str = include_platform!("linux.x86_64.json");
const PLATFORM_LINUX_ARM: &str = include_platform!("linux.aarch64.json");
const PLATFORM_MACOS_X86: &str = include_platform!("macos.x86_64.json");
const PLATFORM_MACOS_ARM: &str = include_platform!("macos.aarch64.json");
const PLATFORM_WINDOWS_X86: &str = include_platform!("windows.x86_64.json");
const PLATFORM_WINDOWS_ARM: &str = include_platform!("windows.aarch64.json");
// 发行版专属
const PLATFORM_LINUX_ARCHLINUX_X86: &str = include_platform!("linux.archlinux.x86_64.json");
const PLATFORM_LINUX_ARCHLINUX_ARM: &str = include_platform!("linux.archlinux.aarch64.json");
const PLATFORM_LINUX_UBUNTU_X86: &str = include_platform!("linux.ubuntu.x86_64.json");
const PLATFORM_LINUX_UBUNTU_ARM: &str = include_platform!("linux.ubuntu.aarch64.json");

fn embedded_mirror_jsons() -> Vec<&'static str> {
    vec![
        MIRROR_PIP,
        MIRROR_NPM,
        MIRROR_YARN,
        MIRROR_PNPM,
        MIRROR_BUN,
        MIRROR_DENO,
        MIRROR_CARGO,
        MIRROR_GO,
        MIRROR_DOCKER,
        MIRROR_K8S_GCR,
        MIRROR_K8S_REGISTRY,
        MIRROR_GHCR,
        MIRROR_QUAY,
        MIRROR_CONDA,
        MIRROR_BREW,
        MIRROR_APT,
        MIRROR_CHOCO,
        MIRROR_NUGET,
        MIRROR_MAVEN,
        MIRROR_GRADLE,
        MIRROR_RUBYGEMS,
        MIRROR_COMPOSER,
        MIRROR_PUB,
        MIRROR_COCOAPODS,
        MIRROR_HUGGINGFACE,
        MIRROR_NODEJS,
        MIRROR_PYTHON,
        MIRROR_RUSTUP,
        MIRROR_VSCODE,
        MIRROR_ANDROID_MAVEN,
        MIRROR_ANDROID_GRADLE,
        MIRROR_SWIFT,
    ]
}

fn embedded_platform_jsons() -> Vec<&'static str> {
    vec![
        // 通用平台
        PLATFORM_LINUX_X86,
        PLATFORM_LINUX_ARM,
        PLATFORM_MACOS_X86,
        PLATFORM_MACOS_ARM,
        PLATFORM_WINDOWS_X86,
        PLATFORM_WINDOWS_ARM,
        // 发行版专属
        PLATFORM_LINUX_ARCHLINUX_X86,
        PLATFORM_LINUX_ARCHLINUX_ARM,
        PLATFORM_LINUX_UBUNTU_X86,
        PLATFORM_LINUX_UBUNTU_ARM,
    ]
}

// ─── 平台检测 ──────────────────────────────────────────────────────

/// 当前 OS: "linux", "macos", "windows"
pub fn current_os() -> &'static str {
    std::env::consts::OS
}

/// 当前架构: "x86_64", "aarch64"
pub fn current_arch() -> &'static str {
    std::env::consts::ARCH
}

/// 检测 Linux 发行版 (读取 /etc/os-release)
pub fn detect_distro() -> Option<String> {
    if current_os() != "linux" {
        return None;
    }

    let content = std::fs::read_to_string("/etc/os-release").ok()?;
    for line in content.lines() {
        if let Some(value) = line.strip_prefix("ID=") {
            let id = value.trim_matches('"').to_lowercase();
            // 归一化
            return Some(match id.as_str() {
                "arch" | "manjaro" | "endeavouros" => "archlinux".to_string(),
                "ubuntu" | "debian" | "linuxmint" | "pop" | "elementary" => "ubuntu".to_string(),
                "centos" | "rhel" | "fedora" | "rocky" | "alma" => "centos".to_string(),
                "alpine" => "alpine".to_string(),
                other => other.to_string(),
            });
        }
    }
    None
}

/// 生成平台标识候选列表 (从最具体到最通用)
fn platform_candidates() -> Vec<String> {
    let os = current_os();
    let arch = current_arch();
    let mut candidates = Vec::new();

    // linux.archlinux.x86_64
    if let Some(distro) = detect_distro() {
        candidates.push(format!("{os}-{distro}-{arch}"));
    }
    // linux.x86_64
    candidates.push(format!("{os}-{arch}"));

    candidates
}

// ─── 加载逻辑 ──────────────────────────────────────────────────────

/// 加载所有镜像源定义（内嵌 + 用户自定义覆盖）
/// 缓存解析结果，避免重复解析 33 个嵌入式 JSON
static MIRROR_DEFS_CACHE: OnceLock<Vec<MirrorDef>> = OnceLock::new();

pub fn load_mirror_defs() -> &'static Vec<MirrorDef> {
    MIRROR_DEFS_CACHE.get_or_init(|| {
        let mut defs: Vec<MirrorDef> = Vec::new();

        for json_str in embedded_mirror_jsons() {
            match serde_json::from_str::<MirrorDef>(json_str) {
                Ok(def) => defs.push(def),
                Err(e) => tracing::warn!("failed to parse embedded mirror config: {}", e),
            }
        }

        // 用户目录覆盖
        if let Some(home) = dirs::home_dir() {
            let user_dir = home.join(".config/nz-switch/mirrors");
            if user_dir.exists() {
                if let Ok(entries) = std::fs::read_dir(&user_dir) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.extension().is_some_and(|e| e == "json") {
                            if let Ok(content) = std::fs::read_to_string(&path) {
                                if let Ok(def) = serde_json::from_str::<MirrorDef>(&content) {
                                    if let Some(existing) =
                                        defs.iter_mut().find(|d| d.tool == def.tool)
                                    {
                                        *existing = def;
                                    } else {
                                        defs.push(def);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        defs
    })
}

/// 缓存当前平台配置
static PLATFORM_DEF_CACHE: OnceLock<PlatformDef> = OnceLock::new();

/// 加载当前平台的配置 (支持 distro 级别匹配)
pub fn load_platform_def() -> &'static PlatformDef {
    PLATFORM_DEF_CACHE.get_or_init(|| {
        let candidates = platform_candidates();

        // 用户目录
        if let Some(home) = dirs::home_dir() {
            for candidate in &candidates {
                let path = home.join(format!(".config/nz-switch/platforms/{candidate}.json"));
                if path.exists() {
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        if let Ok(def) = serde_json::from_str::<PlatformDef>(&content) {
                            return def;
                        }
                    }
                }
            }
        }

        // 内嵌配置 — 先解析再匹配 platform 字段，避免字符串误匹配
        for candidate in &candidates {
            for json_str in embedded_platform_jsons() {
                if let Ok(def) = serde_json::from_str::<PlatformDef>(json_str) {
                    if def.platform == *candidate {
                        return def;
                    }
                }
            }
        }

        // fallback
        serde_json::from_str::<PlatformDef>(embedded_platform_jsons()[0])
            .expect("failed to parse fallback platform config")
    })
}

/// 检查工具是否匹配当前 OS
fn tool_matches_os(def: &MirrorDef) -> bool {
    if def.os.is_empty() {
        return true; // 空 = 所有平台
    }
    def.os.iter().any(|o| o == current_os())
}

/// 检查工具是否匹配当前架构
fn tool_matches_arch(def: &MirrorDef) -> bool {
    if def.arch.is_empty() {
        return true;
    }
    def.arch.iter().any(|a| a == current_arch())
}

/// 根据平台过滤后的镜像源列表
pub fn load_platform_mirrors() -> Vec<(String, Vec<(String, String)>)> {
    let all_defs = load_mirror_defs();
    let platform = load_platform_def();

    let mut result = Vec::new();
    for pt in &platform.tools {
        if let Some(def) = all_defs.iter().find(|d| d.tool == pt.tool) {
            // 双重过滤: 平台配置 + 镜像定义自身的 os/arch
            if !tool_matches_os(def) || !tool_matches_arch(def) {
                continue;
            }

            let mirrors: Vec<(String, String)> = def
                .mirrors
                .iter()
                .filter(|m| m.enabled)
                .filter(|m| m.os.is_empty() || m.os.iter().any(|o| o == current_os()))
                .filter(|m| m.arch.is_empty() || m.arch.iter().any(|a| a == current_arch()))
                .map(|m| (m.name.clone(), m.url.clone()))
                .collect();

            if !mirrors.is_empty() {
                result.push((def.tool.clone(), mirrors));
            }
        }
    }

    result
}

/// 根据工具名查找镜像源定义
pub fn find_mirror_def(tool: &str) -> Option<&'static MirrorDef> {
    load_mirror_defs().iter().find(|d| d.tool == tool)
}

/// 将镜像预设名解析为 URL。如果 source 本身已是 URL 则直接返回。
/// 这是唯一的镜像名→URL 解析入口，apply/env_vars 等模块统一使用此函数。
pub fn resolve_mirror_url(tool: &str, source: &str) -> anyhow::Result<String> {
    if source.starts_with("http://") || source.starts_with("https://") {
        return Ok(source.to_string());
    }
    if let Some(def) = find_mirror_def(tool) {
        if let Some(entry) = def.mirrors.iter().find(|m| m.name == source && m.enabled) {
            return Ok(entry.url.clone());
        }
    }
    Err(anyhow::anyhow!(
        "未找到工具 '{tool}' 的镜像预设 '{source}'，请检查名称是否正确或使用完整 URL"
    ))
}

/// 获取当前平台信息
pub fn get_platform_info() -> (String, String) {
    let platform = load_platform_def();
    (platform.platform.clone(), platform.display_name.clone())
}

/// 工具 → 可执行文件名映射
fn tool_executables(tool: &str) -> Vec<&'static str> {
    match tool {
        "pip" => vec!["pip3", "pip"],
        "npm" => vec!["npm"],
        "yarn" => vec!["yarn"],
        "pnpm" => vec!["pnpm"],
        "bun" => vec!["bun"],
        "deno" => vec!["deno"],
        "cargo" => vec!["cargo"],
        "go" | "goproxy" => vec!["go"],
        "docker" => vec!["docker"],
        "conda" => vec!["conda"],
        "brew" => vec!["brew"],
        "apt" => vec!["apt-get"],
        "choco" => vec!["choco"],
        "nuget" => vec!["nuget"],
        "maven" => vec!["mvn"],
        "gradle" => vec!["gradle"],
        "rubygems" => vec!["gem"],
        "composer" => vec!["composer"],
        "pub" => vec!["dart"],
        "cocoapods" => vec!["pod"],
        "huggingface" => vec!["huggingface-cli", "huggingface"],
        "nodejs" => vec!["node"],
        "python" => vec!["python3", "python"],
        "rustup" => vec!["rustup"],
        "vscode" => vec!["code"],
        "android-maven" | "android-gradle" => vec!["gradle"],
        "swift" => vec!["swift"],
        // K8s/registry 类工具无法简单检测
        "k8s-gcr" | "k8s-registry" | "ghcr" | "quay" => vec!["kubectl"],
        _ => vec![],
    }
}

/// 检查单个工具是否已安装
fn is_tool_installed(tool: &str) -> bool {
    let execs = tool_executables(tool);
    if execs.is_empty() {
        // 无法检测的工具默认显示
        return true;
    }
    execs.iter().any(|exec| which::which(exec).is_ok())
}

/// 获取当前平台已安装的工具列表
pub fn installed_tools() -> Vec<String> {
    load_platform_mirrors()
        .into_iter()
        .filter(|(tool, _)| is_tool_installed(tool))
        .map(|(tool, _)| tool)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_mirror_defs_not_empty() {
        let defs = load_mirror_defs();
        assert!(
            defs.len() >= 30,
            "expected at least 30 mirror defs, got {}",
            defs.len()
        );
    }

    #[test]
    fn test_load_platform_def() {
        let platform = load_platform_def();
        assert!(!platform.tools.is_empty());
    }

    #[test]
    fn test_platform_candidates() {
        let candidates = platform_candidates();
        assert!(!candidates.is_empty());
        // 每个候选格式: os-arch 或 os-distro-arch
        for c in &candidates {
            assert!(
                c.contains('-'),
                "platform candidate should contain '-': {}",
                c
            );
        }
    }

    #[test]
    fn test_load_platform_mirrors() {
        let mirrors = load_platform_mirrors();
        assert!(!mirrors.is_empty());
        for (tool, entries) in &mirrors {
            assert!(!entries.is_empty(), "tool {} should have mirrors", tool);
        }
    }

    #[test]
    fn test_os_filtering() {
        // brew 应在 macos/linux 上出现，windows 上不出现
        let defs = load_mirror_defs();
        let brew = defs.iter().find(|d| d.tool == "brew").unwrap();
        let os = current_os();
        if os == "windows" {
            assert!(!tool_matches_os(brew), "brew should not match windows");
        } else {
            assert!(tool_matches_os(brew), "brew should match {}", os);
        }
    }

    #[test]
    fn test_choco_only_windows() {
        let defs = load_mirror_defs();
        let choco = defs.iter().find(|d| d.tool == "choco").unwrap();
        let os = current_os();
        if os == "windows" {
            assert!(tool_matches_os(choco));
        } else {
            assert!(!tool_matches_os(choco), "choco should not match {}", os);
        }
    }

    #[test]
    fn test_all_mirrors_have_valid_urls() {
        let defs = load_mirror_defs();
        for def in defs.iter() {
            for m in &def.mirrors {
                assert!(
                    m.url.starts_with("http"),
                    "Mirror '{}.{}' has invalid URL: {}",
                    def.tool,
                    m.name,
                    m.url
                );
            }
        }
    }
}
