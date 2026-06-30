pub mod config;
pub mod types;
pub mod test;
pub mod detect;
pub mod apply;
pub mod reset;
pub mod parse;
pub mod paths;
pub mod env_vars;

// Re-exports for backward compatibility
pub use test::{test_mirrors, test_mirrors_concurrent, test_mirrors_streaming};
pub use detect::{detect_current_mirror, detect_all_mirrors};
pub use apply::{apply_mirrors, apply_single_mirror, set_mirror};
pub use paths::{pip_config_path, cargo_config_path};
pub use config::resolve_mirror_url;
pub use reset::{reset_mirror, reset_all_mirrors};
pub use types::*;

use anyhow::Result;
use colored::Colorize;
use crate::format;

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
