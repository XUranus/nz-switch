pub mod apply;
pub mod config;
pub mod detect;
pub mod env_vars;
pub mod parse;
pub mod paths;
pub mod reset;
pub mod test;
pub mod types;

// Re-exports for backward compatibility
pub use apply::{apply_mirrors, apply_single_mirror, set_mirror};
pub use config::resolve_mirror_url;
pub use detect::{detect_all_mirrors, detect_current_mirror};
pub use paths::{cargo_config_path, pip_config_path};
pub use reset::{reset_all_mirrors, reset_mirror};
pub use test::{test_mirrors, test_mirrors_concurrent, test_mirrors_streaming};
pub use types::*;

use crate::format;
use anyhow::Result;
use colored::Colorize;

/// 列出所有支持的镜像源（按平台过滤）
pub fn list_mirrors(tool_filter: Option<&str>) -> Result<()> {
    println!("{}", "🪞 支持的镜像源".cyan().bold());
    println!();

    let all = config::load_platform_mirrors();

    for (tool, mirrors) in &all {
        if let Some(f) = tool_filter {
            if f != tool.as_str() {
                continue;
            }
        }

        let mut table = format::new_table(&["镜像源", "地址"]);
        for (name, url) in mirrors {
            table.add_row(vec![name.cyan().to_string(), url.to_string()]);
        }
        println!("  {}", tool.bold());
        println!("{table}");
    }

    Ok(())
}
