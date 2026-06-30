# nz-switch

> 一键切换中国内地开发环境 — 管理镜像源、代理、环境变量

在中国内地开发受 GFW 影响，需要手动配置大量镜像源和代理。`nz-switch` 帮你一站式管理，一键切换。

## 功能

- 🔀 **Profile 切换** — 一键在「中国内地」和「海外」环境间切换
- 🪞 **镜像源管理** — 支持 pip/npm/cargo/go/docker/conda/brew 等 20+ 工具
- ⚡ **镜像测速** — 自动测试延迟，推荐最快的镜像
- 🌐 **代理管理** — 设置/开启/关闭/测试代理连通性
- 🔑 **环境变量管理** — 管理 GOPROXY/RUSTUP_DIST_SERVER 等
- 🐙 **Git 配置** — GitHub 镜像 + 代理自动配置
- 🩺 **环境诊断** — 检测配置问题，给出修复建议
- 📁 **项目级配置** — 每个项目可覆盖全局设置
- 🔄 **配置导入导出** — JSON 格式，多机同步
- ⌨️ **Shell 补全** — 支持 bash/zsh/fish/powershell
- 🖥️ **GUI 界面** — Tauri v2 桌面应用，可视化管理所有功能

## 安装

```bash
# 从源码编译
git clone https://github.com/yourname/nz-switch.git
cd nz-switch
cargo install --path .
```

### GUI 应用

```bash
cd gui
npm install
cargo tauri dev    # 开发模式
cargo tauri build  # 构建安装包
```

## 快速开始

```bash
# 1. 初始化配置
nz-switch init

# 2. 切换到中国内地环境 (一键配置所有)
nz-switch switch cn

# 3. 查看当前状态
nz-switch status

# 4. 诊断环境问题
nz-switch doctor
```

## 命令一览

| 命令 | 功能 |
|------|------|
| `nz-switch init` | 初始化全局配置 |
| `nz-switch init --local` | 初始化项目级配置 (.nz-switch.toml) |
| `nz-switch switch cn` | 切换到中国内地环境 |
| `nz-switch switch global` | 切换到海外环境 |
| `nz-switch status` | 查看当前环境状态 |
| `nz-switch doctor` | 诊断环境问题 |

### 镜像源管理

```bash
nz-switch mirror list                     # 列出所有支持的镜像源
nz-switch mirror test                     # 测试所有镜像源速度
nz-switch mirror test --tool pip          # 只测试 pip 镜像
nz-switch mirror set pip https://mirrors.aliyun.com/pypi/simple
nz-switch mirror reset cargo              # 恢复默认
```

### 代理管理

```bash
nz-switch proxy set http://127.0.0.1:7890  # 设置代理地址
nz-switch proxy on                          # 开启代理
nz-switch proxy off                         # 关闭代理
nz-switch proxy test                        # 测试国内外连通性
```

### 环境变量

```bash
nz-switch env show                # 查看当前 profile 的环境变量
nz-switch env set KEY VALUE       # 设置环境变量
nz-switch env unset KEY           # 删除环境变量
nz-switch env proxy               # 列出 proxy 相关环境变量
```

### 配置管理

```bash
nz-switch config show             # 显示配置
nz-switch config path             # 显示配置文件路径
nz-switch config edit             # 用编辑器打开
nz-switch config export -o backup.json  # 导出为 JSON
nz-switch config import backup.json     # 导入配置
nz-switch config import backup.json --merge  # 合并导入
nz-switch config reset            # 重置为默认
```

### 项目级配置

```bash
cd my-project
nz-switch init --local            # 创建 .nz-switch.toml
nz-switch local show              # 查看项目配置
nz-switch local path              # 显示配置路径
```

### Shell 补全

```bash
nz-switch completions zsh > ~/.zfunc/_nz-switch
nz-switch completions bash > ~/.local/share/bash-completion/completions/nz-switch
nz-switch completions fish > ~/.config/fish/completions/nz-switch.fish
```

## 配置文件

### 全局配置

路径: `~/.config/nz-switch/config.toml`

```toml
current_profile = "cn"

[profiles.cn]
display_name = "中国内地"

[profiles.cn.env]
GOPROXY = "https://goproxy.cn,direct"
RUSTUP_DIST_SERVER = "https://mirrors.ustc.edu.cn/rust-static"

[profiles.cn.mirrors]
pip = "https://pypi.tuna.tsinghua.edu.cn/simple"
npm = "https://registry.npmmirror.com"
# cargo = "ustc"  # 可选: 手动启用 cargo 镜像

[profiles.cn.proxy]
address = "http://127.0.0.1:7890"
proxy_type = "http"

[profiles.cn.git]
github_mirror = "https://ghproxy.com/"
proxy = "http://127.0.0.1:7890"
```

### 项目级配置

在项目根目录创建 `.nz-switch.toml`，可覆盖全局设置:

```toml
base_profile = "cn"

[mirrors]
npm = "https://registry.npmmirror.com"
pip = "https://mirrors.aliyun.com/pypi/simple"

[env]
NODE_ENV = "development"
```

## 支持的镜像源

| 工具 | 支持的镜像 |
|------|-----------|
| **pip** | 清华、阿里云、ustc、豆瓣、华为 |
| **npm** | npmmirror、腾讯、华为 |
| **cargo** | ustc、清华、rsproxy |
| **go** | goproxy.cn、goproxy.io |
| **docker** | 阿里云、腾讯、华为、163、DaoCloud |
| **conda** | 清华、ustc、阿里云 |
| **brew** | 清华、ustc |
| **rustup** | ustc、清华 |
| **nodejs** | npmmirror |
| **huggingface** | hf-mirror |

## Profile 系统

内置两个 Profile:

- **cn** — 中国内地环境，配置国内镜像 + 代理
- **global** — 海外环境，恢复默认配置

切换到 `global` 时自动清除国内镜像配置文件和环境变量。

## 开发

### CLI

```bash
cargo build          # 编译
cargo test           # 运行测试
cargo clippy         # 代码检查
cargo run -- --help  # 查看帮助
```

### GUI (Tauri v2)

```bash
cd gui
npm install          # 安装前端依赖
cargo tauri dev      # 启动开发模式
cargo tauri build    # 构建生产包
```

## License

MIT
