// 环境变量映射集中管理
//
// 每个工具的环境变量名在此统一定义，apply/detect/reset 共用。

use anyhow::Result;

use super::config;

// ─── 环境变量注册表 ──────────────────────────────────────────────────

/// 工具 → 环境变量名列表（用于检测和重置）
/// 返回的 slice 中第一个元素为主变量（用于检测）
pub fn env_var_names(tool: &str) -> &'static [&'static str] {
    match tool {
        "go" | "goproxy" => &["GOPROXY"],
        "nodejs" => &["NVM_NODEJS_ORG_MIRROR"],
        "python" => &["PIP_INDEX_URL"],
        "bun" => &["BUN_CONFIG_REGISTRY"],
        "deno" => &["DENO_INSTALL_MIRROR"],
        "huggingface" => &["HF_ENDPOINT"],
        "rustup" => &["RUSTUP_DIST_SERVER", "RUSTUP_UPDATE_ROOT"],
        "brew" => &[
            "HOMEBREW_BREW_GIT_REMOTE",
            "HOMEBREW_CORE_GIT_REMOTE",
            "HOMEBREW_API_DOMAIN",
            "HOMEBREW_BOTTLE_DOMAIN",
        ],
        "pub" => &["PUB_HOSTED_URL", "FLUTTER_STORAGE_BASE_URL"],
        _ => &[],
    }
}

/// 获取工具的主环境变量名（用于镜像检测）
pub fn primary_env_var(tool: &str) -> Option<&'static str> {
    env_var_names(tool).first().copied()
}

// ─── 环境变量条目生成 ──────────────────────────────────────────────

/// 获取 env 类型工具的环境变量映射
/// 优先从 MirrorEntry 的 env_vars 字段读取（数据驱动），回退到硬编码
pub fn env_var_entries(tool: &str, source: &str) -> Result<Vec<(String, String)>> {
    // 优先从 JSON 定义读取
    if let Some(entries) = lookup_env_vars_from_def(tool, source) {
        if !entries.is_empty() {
            return Ok(entries);
        }
    }

    // 如果 source 是预设名，先解析为 URL
    let resolved = config::resolve_mirror_url(tool, source)?;

    Ok(match tool {
        "nodejs" => vec![("NVM_NODEJS_ORG_MIRROR".into(), resolved)],
        "python" => vec![("PIP_INDEX_URL".into(), resolved)],
        "rustup" => {
            let base = resolved.trim_end_matches('/');
            vec![
                ("RUSTUP_DIST_SERVER".into(), base.to_string()),
                ("RUSTUP_UPDATE_ROOT".into(), format!("{base}/rustup")),
            ]
        }
        "bun" => vec![("BUN_CONFIG_REGISTRY".into(), resolved)],
        "deno" => vec![("DENO_INSTALL_MIRROR".into(), resolved)],
        "huggingface" => vec![("HF_ENDPOINT".into(), resolved)],
        "go" | "goproxy" => {
            let val = if source == "goproxy.cn" || source == "goproxy" {
                "https://goproxy.cn,direct".to_string()
            } else if source == "mirrors.aliyun.com" {
                "https://mirrors.aliyun.com/go-proxy,direct".to_string()
            } else {
                resolved
            };
            vec![("GOPROXY".into(), val)]
        }
        _ => vec![],
    })
}

// ─── 内部辅助 ──────────────────────────────────────────────────────

/// 从镜像定义的 env_vars 字段查找环境变量映射
fn lookup_env_vars_from_def(tool: &str, source: &str) -> Option<Vec<(String, String)>> {
    let def = config::find_mirror_def(tool)?;
    let entry = def.mirrors.iter().find(|m| {
        m.enabled && (m.name == source || m.url == source)
    })?;
    let env_vars = entry.env_vars.as_ref()?;
    if env_vars.is_empty() {
        return None;
    }
    Some(env_vars.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
}
