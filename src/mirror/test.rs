// 镜像源测速

use anyhow::Result;
use colored::Colorize;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::net::TcpStream;
use tokio::sync::Semaphore;
use tokio::time::timeout;

use super::config;
use super::types::{MirrorLatency, MirrorSingleResult, MirrorTestResult};
use crate::format;

// ─── 常量 ──────────────────────────────────────────────────────────

/// 最大并发 TCP 测速数
const MAX_CONCURRENT_PINGS: usize = 16;
/// 测速结果通道缓冲大小
const PING_CHANNEL_BUFFER: usize = 64;
/// 单次 TCP 连接超时
const TCP_PING_TIMEOUT: Duration = Duration::from_secs(3);

/// 构建 tokio 运行时（各同步包装函数共用）
fn build_tokio_runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("failed to build tokio runtime")
}

/// 收集所有待测速任务（各 async 函数共用）
fn collect_tasks(tool_filter: Option<&str>) -> Vec<(String, String, String)> {
    let all = config::load_platform_mirrors();
    let mut tasks = Vec::new();
    for (tool, mirrors) in &all {
        if let Some(f) = tool_filter {
            if f != tool.as_str() {
                continue;
            }
        }
        for (name, url) in mirrors {
            tasks.push((tool.clone(), name.clone(), url.clone()));
        }
    }
    tasks
}

/// 测试镜像源速度（CLI 用，async TCP 延迟）
pub fn test_mirrors(tool_filter: Option<&str>) -> Result<()> {
    let rt = build_tokio_runtime();
    rt.block_on(test_mirrors_async(tool_filter))
}

async fn test_mirrors_async(tool_filter: Option<&str>) -> Result<()> {
    println!("{}", "⚡ 镜像源测速 (TCP 延迟)".cyan().bold());
    println!();

    let all = config::load_platform_mirrors();

    for (tool, mirrors) in &all {
        if let Some(f) = tool_filter {
            if f != tool.as_str() {
                continue;
            }
        }

        let mut table = format::new_table(&["状态", "镜像源", "延迟", "地址"]);
        let mut results: Vec<(&String, &String, Option<Duration>)> = Vec::new();

        for (name, url) in mirrors {
            let latency = ping_url_tcp(url).await;
            results.push((name, url, latency));

            let (icon, ms_str) = match latency {
                Some(d) => {
                    let ms = d.as_millis() as u64;
                    (format::latency_icon(ms), format!("{}ms", ms))
                }
                None => ("⏱️".to_string(), "超时".red().to_string()),
            };

            table.add_row(vec![icon, name.cyan().to_string(), ms_str, url.to_string()]);
        }

        // 推荐最快
        let fastest = results
            .iter()
            .filter_map(|(name, url, d)| d.map(|dur| (*name, *url, dur)))
            .min_by_key(|(_, _, d)| *d);

        println!("  {}", tool.bold());
        println!("{table}");

        if let Some((name, url, d)) = fastest {
            println!(
                "  {} 推荐: {} ({}, {}ms)",
                format::tip(),
                name.bold(),
                url,
                d.as_millis()
            );
        }

        println!();
    }

    Ok(())
}

/// 并发测速所有镜像源（Tauri 用，批量返回）— async 真流式
pub fn test_mirrors_concurrent(tool_filter: Option<&str>) -> Vec<MirrorTestResult> {
    let rt = build_tokio_runtime();
    rt.block_on(test_mirrors_concurrent_async(tool_filter))
}

async fn test_mirrors_concurrent_async(tool_filter: Option<&str>) -> Vec<MirrorTestResult> {
    let all = config::load_platform_mirrors();
    let tasks = collect_tasks(tool_filter);

    let semaphore = std::sync::Arc::new(Semaphore::new(MAX_CONCURRENT_PINGS));
    let (tx, mut rx) = tokio::sync::mpsc::channel(PING_CHANNEL_BUFFER);

    // spawn 所有任务，用 semaphore 控制并发
    for (tool, name, url) in tasks {
        let sem = semaphore.clone();
        let tx = tx.clone();
        tokio::spawn(async move {
            let _permit = sem.acquire().await.unwrap();
            let latency = ping_url_tcp(&url).await;
            let _ = tx.send((tool, name, url, latency)).await;
        });
    }
    drop(tx);

    // 收集所有结果
    let mut raw: HashMap<String, Vec<MirrorLatency>> = HashMap::new();
    while let Some((tool, name, url, latency)) = rx.recv().await {
        raw.entry(tool).or_default().push(MirrorLatency {
            name,
            url,
            latency_ms: latency.map(|d| d.as_millis() as u64),
        });
    }

    // 按工具排序，计算推荐
    let mut results: Vec<MirrorTestResult> = Vec::new();
    for (tool, _mirrors) in &all {
        if let Some(f) = tool_filter {
            if f != tool.as_str() {
                continue;
            }
        }
        if let Some(mut latencies) = raw.remove(tool) {
            latencies.sort_by_key(|l| l.latency_ms.unwrap_or(u64::MAX));
            let recommended = latencies.first().and_then(|l| {
                if l.latency_ms.is_some() {
                    Some(l.name.clone())
                } else {
                    None
                }
            });
            results.push(MirrorTestResult {
                tool: tool.clone(),
                results: latencies,
                recommended,
            });
        }
    }

    results
}

/// 流式并发测速：每完成一个镜像就回调一次（真流式，async）
pub fn test_mirrors_streaming<F>(tool_filter: Option<&str>, on_result: F)
where
    F: FnMut(MirrorSingleResult) + Send + 'static,
{
    let rt = build_tokio_runtime();
    rt.block_on(test_mirrors_streaming_async(tool_filter, on_result))
}

async fn test_mirrors_streaming_async<F>(tool_filter: Option<&str>, mut on_result: F)
where
    F: FnMut(MirrorSingleResult) + Send + 'static,
{
    let tasks = collect_tasks(tool_filter);

    let semaphore = std::sync::Arc::new(Semaphore::new(MAX_CONCURRENT_PINGS));
    let (tx, mut rx) = tokio::sync::mpsc::channel(PING_CHANNEL_BUFFER);

    // spawn 所有任务，每个完成后立即发送结果（真流式）
    for (tool, name, url) in tasks {
        let sem = semaphore.clone();
        let tx = tx.clone();
        tokio::spawn(async move {
            let _permit = sem.acquire().await.unwrap();
            let latency = ping_url_tcp(&url).await;
            let _ = tx
                .send(MirrorSingleResult {
                    tool,
                    name,
                    url,
                    latency_ms: latency.map(|d| d.as_millis() as u64),
                })
                .await;
        });
    }
    drop(tx);

    // 每收到一个结果就立即回调（真流式：不再等批次）
    while let Some(result) = rx.recv().await {
        on_result(result);
    }
}

/// 异步测量 TCP 连接延迟（仅 TCP 握手，排除 TLS/HTTP 开销）
async fn ping_url_tcp(url: &str) -> Option<Duration> {
    // 从 URL 中提取 host 和 port
    // 清理 Go GOPROXY 协议后缀（如 ",direct"）
    let clean_url = url.split(',').next().unwrap_or(url);
    let (host, port) = parse_host_port(clean_url)?;
    let addr = format!("{host}:{port}");

    let start = Instant::now();
    let result = timeout(TCP_PING_TIMEOUT, TcpStream::connect(&addr)).await;
    match result {
        Ok(Ok(_stream)) => Some(start.elapsed()),
        _ => None,
    }
}

/// 从 URL 中提取 host 和默认端口 (443 for https, 80 for http)
fn parse_host_port(url: &str) -> Option<(&str, u16)> {
    let (scheme_end, default_port) = if let Some(rest) = url.strip_prefix("https://") {
        (rest, 443)
    } else if let Some(rest) = url.strip_prefix("http://") {
        (rest, 80)
    } else {
        return None;
    };

    // host 部分: 到第一个 '/' 或 ':' 或结尾
    let host_end = scheme_end.find('/').unwrap_or(scheme_end.len());
    let host_part = &scheme_end[..host_end];

    // 检查是否有显式端口
    if let Some(colon_pos) = host_part.find(':') {
        let port: u16 = host_part[colon_pos + 1..].parse().ok()?;
        Some((&host_part[..colon_pos], port))
    } else {
        Some((host_part, default_port))
    }
}
