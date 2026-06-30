//! CLI 表格输出工具模块

use comfy_table::{Table, ContentArrangement, presets::UTF8_FULL, modifiers::UTF8_ROUND_CORNERS};
use colored::Colorize;

/// 创建带圆角 UTF-8 边框的表格，自动适配终端宽度
pub fn new_table(headers: &[&str]) -> Table {
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_content_arrangement(ContentArrangement::DynamicFullWidth)
        .set_header(headers.iter().map(|h| h.bold().to_string()));
    table
}

/// 绿色成功状态 emoji
pub fn ok() -> String {
    "✅".green().to_string()
}

/// 黄色警告状态 emoji
pub fn warn() -> String {
    "⚠️".yellow().to_string()
}

/// 红色错误状态 emoji
pub fn err() -> String {
    "❌".red().to_string()
}

/// 蓝色信息 emoji
pub fn info() -> String {
    "ℹ".blue().to_string()
}

/// 绿色推荐 emoji
pub fn tip() -> String {
    "💡".green().to_string()
}

/// 根据延迟返回交通灯 emoji
pub fn latency_icon(ms: u64) -> String {
    if ms < 100 {
        "🟢".to_string()
    } else if ms < 300 {
        "🟡".to_string()
    } else {
        "🔴".to_string()
    }
}

/// 截断过长字符串到指定宽度，超出部分显示 "..."
pub fn truncate(s: &str, max_len: usize) -> String {
    // 尊重 CJK 字符宽度：CJK 字符占 2 列，ASCII 占 1 列
    let mut width = 0;
    for (i, ch) in s.char_indices() {
        let w = if ch.is_ascii() { 1 } else { 2 };
        if width + w > max_len.saturating_sub(3) {
            return format!("{}...", &s[..i]);
        }
        width += w;
    }
    s.to_string()
}
