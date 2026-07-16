// 解析工具函数

/// 从 INI 格式中提取 [section] 下的 key = value
pub fn parse_ini_value(content: &str, section: &str, key: &str) -> Option<String> {
    let mut in_section = false;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed == format!("[{section}]") {
            in_section = true;
            continue;
        }
        if in_section && trimmed.starts_with('[') {
            break; // 下一个 section
        }
        if in_section && trimmed.starts_with(key) {
            // 确保 key 后面紧跟 = 或空白，避免 "index-url" 匹配 "index-url-suffix"
            let after_key = &trimmed[key.len()..];
            if after_key.starts_with('=')
                || after_key.starts_with(' ')
                || after_key.starts_with('\t')
                || after_key.is_empty()
            {
                let val = after_key.trim().strip_prefix('=').unwrap_or("").trim();
                if !val.is_empty() {
                    return Some(val.to_string());
                }
            }
        }
    }
    None
}

/// 从 key=value 格式中提取 (如 .npmrc)
pub fn parse_key_eq_value(content: &str, key: &str) -> Option<String> {
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with(key) {
            let val = trimmed.strip_prefix(key)?.trim().strip_prefix('=')?.trim();
            if !val.is_empty() {
                return Some(val.to_string());
            }
        }
    }
    None
}

/// 解析 maven settings.xml 中 <mirror> 块的 <url>
pub fn parse_maven_mirror_url(content: &str) -> Option<String> {
    // 找到 <mirror>...</mirror> 块，提取其中的 <url>
    let mut rest = content;
    while let Some(mirror_start) = rest.find("<mirror>") {
        let mirror_end = rest[mirror_start..].find("</mirror>")?;
        let block = &rest[mirror_start..mirror_start + mirror_end + "</mirror>".len()];
        // 在 mirror 块中找 <url>
        if let Some(url_start) = block.find("<url>") {
            let after = &block[url_start + "<url>".len()..];
            if let Some(url_end) = after.find("</url>") {
                let val = after[..url_end].trim();
                if !val.is_empty() {
                    return Some(val.to_string());
                }
            }
        }
        rest = &rest[mirror_start + mirror_end..];
    }
    None
}

/// 解析 apt sources.list，提取所有镜像 base URL
pub fn parse_apt_sources(path: &str) -> Vec<String> {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return vec![],
    };
    let mut urls = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('#') {
            continue;
        }
        if trimmed.starts_with("deb ") || trimmed.starts_with("deb\t") {
            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            if parts.len() >= 2 {
                let url = parts[1];
                if url.starts_with("http") {
                    urls.push(url.trim_end_matches('/').to_string());
                }
            }
        }
    }
    urls
}

#[cfg(test)]
mod tests {
    use super::*;

    // ─── parse_ini_value ─────────────────────────────────────────

    #[test]
    fn test_ini_basic() {
        let content = "[global]\nindex-url = https://pypi.tuna.tsinghua.edu.cn/simple\ntrusted-host = pypi.tuna.tsinghua.edu.cn\n";
        assert_eq!(
            parse_ini_value(content, "global", "index-url"),
            Some("https://pypi.tuna.tsinghua.edu.cn/simple".into())
        );
    }

    #[test]
    fn test_ini_missing_section() {
        let content = "[other]\nindex-url = https://example.com\n";
        assert_eq!(parse_ini_value(content, "global", "index-url"), None);
    }

    #[test]
    fn test_ini_missing_key() {
        let content = "[global]\ntrusted-host = example.com\n";
        assert_eq!(parse_ini_value(content, "global", "index-url"), None);
    }

    #[test]
    fn test_ini_key_prefix_no_match() {
        // "index-url-suffix" 不应匹配 "index-url"
        let content =
            "[global]\nindex-url-suffix = https://example.com\nindex-url = https://real.com\n";
        assert_eq!(
            parse_ini_value(content, "global", "index-url"),
            Some("https://real.com".into())
        );
    }

    #[test]
    fn test_ini_empty_value() {
        let content = "[global]\nindex-url =\n";
        assert_eq!(parse_ini_value(content, "global", "index-url"), None);
    }

    // ─── parse_key_eq_value ─────────────────────────────────────

    #[test]
    fn test_key_eq_basic() {
        let content = "registry=https://registry.npmmirror.com/\nother=value\n";
        assert_eq!(
            parse_key_eq_value(content, "registry"),
            Some("https://registry.npmmirror.com/".into())
        );
    }

    #[test]
    fn test_key_eq_missing() {
        let content = "other=value\n";
        assert_eq!(parse_key_eq_value(content, "registry"), None);
    }

    #[test]
    fn test_key_eq_empty() {
        let content = "registry=\n";
        assert_eq!(parse_key_eq_value(content, "registry"), None);
    }

    // ─── parse_maven_mirror_url ──────────────────────────────────

    #[test]
    fn test_maven_basic() {
        let content = "<settings>\n<mirrors>\n<mirror>\n<id>mirror</id>\n<mirrorOf>central</mirrorOf>\n<url>https://maven.aliyun.com/repository/public</url>\n</mirror>\n</mirrors>\n</settings>";
        assert_eq!(
            parse_maven_mirror_url(content),
            Some("https://maven.aliyun.com/repository/public".into())
        );
    }

    #[test]
    fn test_maven_no_mirror() {
        let content = "<settings>\n<profiles>\n</profiles>\n</settings>";
        assert_eq!(parse_maven_mirror_url(content), None);
    }

    #[test]
    fn test_maven_no_url() {
        let content = "<mirror>\n<id>mirror</id>\n<mirrorOf>central</mirrorOf>\n</mirror>";
        assert_eq!(parse_maven_mirror_url(content), None);
    }

    // ─── parse_apt_sources ───────────────────────────────────────

    #[test]
    fn test_apt_sources_basic() {
        let dir = std::env::temp_dir().join("nz-switch-test-apt-parse");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        let path = dir.join("sources.list");
        std::fs::write(&path, "deb http://archive.ubuntu.com/ubuntu/ jammy main\ndeb http://security.ubuntu.com/ubuntu/ jammy-security main\n# comment\n").unwrap();

        let urls = parse_apt_sources(path.to_str().unwrap());
        assert_eq!(urls.len(), 2);
        assert_eq!(urls[0], "http://archive.ubuntu.com/ubuntu");
        assert_eq!(urls[1], "http://security.ubuntu.com/ubuntu");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_apt_sources_empty() {
        let dir = std::env::temp_dir().join("nz-switch-test-apt-empty");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        let path = dir.join("sources.list");
        std::fs::write(&path, "# only comments\n").unwrap();

        let urls = parse_apt_sources(path.to_str().unwrap());
        assert!(urls.is_empty());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_apt_sources_nonexistent() {
        let urls = parse_apt_sources("/nonexistent/path/sources.list");
        assert!(urls.is_empty());
    }
}
