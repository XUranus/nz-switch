use clap::{Parser, Subcommand, ValueEnum};

/// nz-switch — 一键切换中国内地开发环境
///
/// 管理镜像源、代理、环境变量，让你在 GFW 内外都能顺畅开发。
#[derive(Parser, Debug)]
#[command(name = "nz-switch", version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// 交互式初始化配置
    Init {
        /// 初始化项目级配置 (.nz-switch.toml)
        #[arg(long)]
        local: bool,
    },

    /// 切换开发环境 profile
    Switch {
        /// Profile 名称 (cn / global / 自定义名称)
        profile: String,

        /// 仅预览变更，不实际执行
        #[arg(long)]
        dry_run: bool,
    },

    /// 查看当前环境状态
    Status,

    /// 管理配置文件
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },

    /// 镜像源管理
    Mirror {
        #[command(subcommand)]
        action: MirrorAction,
    },

    /// 代理管理
    Proxy {
        #[command(subcommand)]
        action: ProxyAction,
    },

    /// 环境变量管理
    Env {
        #[command(subcommand)]
        action: EnvAction,
    },

    /// 诊断环境问题
    Doctor,

    /// DNS 管理
    Dns {
        #[command(subcommand)]
        action: DnsAction,
    },

    /// 项目级配置管理 (.nz-switch.toml)
    Local {
        #[command(subcommand)]
        action: LocalAction,
    },

    /// 生成 Shell 自动补全脚本
    Completions {
        /// 目标 shell
        #[arg(value_enum)]
        shell: Shell,
    },

    /// 管理自定义 Profile
    Profile {
        #[command(subcommand)]
        action: ProfileAction,
    },
}

#[derive(Debug, Clone, ValueEnum)]
#[allow(clippy::enum_variant_names)]
pub enum Shell {
    Bash,
    Zsh,
    Fish,
    PowerShell,
    Elvish,
}

#[derive(Subcommand, Debug)]
pub enum ConfigAction {
    /// 显示当前配置
    Show,
    /// 显示配置文件路径
    Path,
    /// 用编辑器打开配置文件
    Edit,
    /// 导出配置到 JSON 文件
    Export {
        /// 输出文件路径 (默认: stdout)
        #[arg(short, long)]
        output: Option<String>,
    },
    /// 从 JSON 文件导入配置
    Import {
        /// 输入文件路径
        file: String,
        /// 合并到现有配置 (而非替换)
        #[arg(long)]
        merge: bool,
    },
    /// 重置为默认配置
    Reset,
}

#[derive(Subcommand, Debug)]
pub enum MirrorAction {
    /// 列出所有支持的镜像源
    List {
        /// 指定工具 (pip, npm, cargo, ...)
        #[arg(short, long)]
        tool: Option<String>,
    },

    /// 测试镜像源速度
    Test {
        /// 指定工具 (pip, npm, cargo, ...)
        #[arg(short, long)]
        tool: Option<String>,
    },

    /// 设置某个工具的镜像源
    Set {
        /// 工具名 (pip, npm, cargo, ...)
        tool: String,
        /// 镜像源名称或 URL
        source: String,
    },

    /// 恢复某个工具的默认镜像源
    Reset {
        /// 工具名
        tool: String,
    },
}

#[derive(Subcommand, Debug)]
pub enum ProxyAction {
    /// 设置代理地址
    Set {
        /// 代理地址 (如 http://127.0.0.1:7890)
        address: String,
    },

    /// 开启代理环境变量
    On,

    /// 关闭代理环境变量
    Off,

    /// 测试代理连通性
    Test,
}

#[derive(Subcommand, Debug)]
pub enum EnvAction {
    /// 显示当前 profile 的环境变量
    Show,

    /// 设置环境变量 (写入当前 profile 配置)
    Set {
        /// 环境变量名
        key: String,
        /// 环境变量值
        value: String,
    },

    /// 删除环境变量 (从当前 profile 配置中移除)
    Unset {
        /// 环境变量名
        key: String,
    },

    /// 列出当前 shell 中所有 proxy 相关的环境变量
    Proxy,
}

#[derive(Subcommand, Debug)]
pub enum LocalAction {
    /// 显示项目级配置
    Show,

    /// 创建项目级配置文件
    Init,

    /// 显示项目级配置文件路径
    Path,
}

#[derive(Subcommand, Debug)]
pub enum DnsAction {
    /// 显示当前 DNS 配置
    Show,

    /// 列出所有 DNS 预设
    List,

    /// 设置 DNS (预设名或 IP 地址)
    Set {
        /// DNS 预设名 (alibaba/114/tencent/google/cloudflare) 或 IP 地址
        source: String,
    },
}

#[derive(Subcommand, Debug)]
pub enum ProfileAction {
    /// 列出所有 profile
    List,

    /// 创建新的自定义 profile
    Create {
        /// Profile 名称
        name: String,
    },

    /// 删除自定义 profile (不能删除内置 profile)
    Delete {
        /// Profile 名称
        name: String,
    },
}
