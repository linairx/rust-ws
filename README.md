# Rust-WS

一个用 Rust 编写的高性能 WebSocket 代理服务器，支持 VLESS、Trojan、Shadowsocks 协议。

## 项目架构

本项目采用 **Rust Workspace** 架构，分为两个独立的 crate：

```
rust-ws/
├── rust-ws-core/          # 核心共享库
│   ├── src/
│   │   ├── lib.rs         # 库入口
│   │   ├── error.rs       # 错误类型
│   │   ├── utils.rs       # 工具函数
│   │   ├── subscription.rs # 订阅生成
│   │   └── proxy/         # 协议解析
│   │       ├── vless.rs
│   │       ├── trojan.rs
│   │       └── shadowsocks.rs
│   └── Cargo.toml
└── rust-ws-proxy/         # VPS WebSocket 代理服务
    ├── src/
    │   ├── main.rs        # 入口
    │   ├── config.rs      # 配置
    │   └── server/
    │       ├── handlers.rs # HTTP 处理
    │       └── ws.rs       # WebSocket 处理
    ├── static/
    └── Cargo.toml
```

## 功能特性

- 🔌 **多协议支持**: VLESS、Trojan、Shadowsocks
- 🌐 **WebSocket 传输**: 基于 WebSocket 的代理传输
- 📡 **订阅生成**: 自动生成客户端订阅链接
- 🚀 **高性能**: 基于 Tokio 异步运行时
- 🐳 **Docker 支持**: 开箱即用的容器化部署

## 快速开始

### 构建

```bash
# 构建核心库
cargo build -p rust-ws-core --release

# 构建 VPS 代理服务
cargo build -p rust-ws-proxy --release

# 设置环境变量
export UUID=your-uuid-here
export DOMAIN=example.com
export PORT=3000

# 运行
./target/release/rust-ws-proxy
```

## 环境变量配置

### rust-ws-proxy (VPS)

| 变量 | 默认值 | 说明 |
|------|--------|------|
| `UUID` | `7bd180e8-1142-4387-93f5-03e8d750a896` | 节点 UUID |
| `PORT` | `3000` | 服务监听端口 |
| `DOMAIN` | (空) | 域名，为空则使用 IP |
| `SUB_PATH` | `sub` | 订阅路径 |
| `NAME` | (空) | 节点名称前缀 |
| `WS_PATH` | `7bd180e8` | WebSocket 路径 |
| `AUTO_ACCESS` | `false` | 自动保活 |
| `DEBUG` | `false` | 调试模式 |
| `ALLOW_SHADOWSOCKS` | `false` | 是否允许 Shadowsocks 首包解析（公网建议关闭） |

## API 端点

### rust-ws-proxy

| 端点 | 方法 | 说明 |
|------|------|------|
| `/` | GET | 首页（世界时钟） |
| `/health` | GET | 健康检查 |
| `/{sub_path}` | GET | 订阅链接（Base64 编码） |
| `/{ws_path}` | WS | WebSocket 代理入口 |

## 协议支持

### VLESS

首包格式：
```
+------+----------+----------+--------+------+----------+------+
| Ver  |   UUID   | Addons   | Cmd    | Type |  Addr    | Port |
| 1B   |   16B    | 1B+Var   | 1B     | 1B   |  Var     | 2B   |
+------+----------+----------+--------+------+----------+------+
```

### Trojan

首包格式：
```
+------------------+------+------+
|   Password Hash  | CRLF | Cmd  |
|      56B         |  2B  | 1B   |
+------------------+------+------+
```

### Shadowsocks

首包格式：
```
+------+----------+------+
| Type |   Addr   | Port |
| 1B   |   Var    | 2B   |
+------+----------+------+
```

## 域名过滤

以下域名会被自动屏蔽（测速网站）：

- speedtest.net
- fast.com
- speedtest.cn
- speed.cloudflare.com
- speedof.me

## 技术栈

- [Tokio](https://tokio.rs/) - 异步运行时
- [Axum](https://github.com/tokio-rs/axum) - Web 框架
- [Tower](https://github.com/tower-rs/tower) - 中间件
- [Reqwest](https://github.com/seanmonstar/reqwest) - HTTP 客户端

## GitHub 自动构建镜像（GHCR）

项目已提供 GitHub Actions 工作流：`.github/workflows/docker-image.yml`。

- 触发条件：
  - push 到 `main`
  - push `v*` 标签（例如 `v1.0.0`）
  - 手动触发（workflow_dispatch）
- 推送地址：`ghcr.io/<owner>/<repo>`（当前仓库默认会推到 `ghcr.io/<owner>/rust-ws`）
- 支持架构：`linux/amd64`、`linux/arm64`
- CI 使用 BuildKit GHA cache + Cargo registry/git cache mount，加速第三方依赖编译

使用时请确保仓库 **Settings → Actions → General** 允许工作流读取并写入 packages（`GITHUB_TOKEN` 权限）。
如果组织策略限制 `GITHUB_TOKEN` 发布 GHCR，可在仓库 Secrets 配置 `GHCR_USERNAME` / `GHCR_TOKEN`（PAT 需包含 `write:packages`）。

## 许可证

MIT License

## 致谢

- 灵感来源: [python-ws](https://github.com/eooce/python-ws)
