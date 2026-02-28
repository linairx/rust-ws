# Rust-WS

一个用 Rust 编写的高性能 WebSocket 代理服务器，支持 VLESS、Trojan、Shadowsocks 协议。

## 功能特性

- 🔌 **多协议支持**: VLESS、Trojan、Shadowsocks
- 🌐 **WebSocket 传输**: 基于 WebSocket 的代理传输
- 📡 **订阅生成**: 自动生成客户端订阅链接
- 🚀 **高性能**: 基于 Tokio 异步运行时
- 🐳 **Docker 支持**: 开箱即用的容器化部署
- ⚡ **低内存占用**: 约 10MB 内存使用

## 快速开始

### Docker 部署（推荐）

```bash
# 使用 docker-compose
docker-compose up -d

# 或直接运行
docker run -d \
  -p 3000:3000 \
  -e UUID=your-uuid-here \
  -e DOMAIN=example.com \
  ghcr.io/linairx/rust-ws:latest
```

### 从源码构建

```bash
# 克隆仓库
git clone https://github.com/linairx/rust-ws.git
cd rust-ws

# 构建
cargo build --release

# 运行
./target/release/rust-ws
```

### 环境变量配置

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

## API 端点

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

## 项目结构

```
rust-ws/
├── src/
│   ├── main.rs              # 入口
│   ├── config.rs            # 配置管理
│   ├── error.rs             # 错误类型
│   ├── proxy/
│   │   ├── handler.rs       # WebSocket 处理
│   │   ├── vless.rs         # VLESS 协议
│   │   ├── trojan.rs        # Trojan 协议
│   │   └── shadowsocks.rs   # Shadowsocks 协议
│   ├── http/
│   │   ├── handlers.rs      # HTTP 处理器
│   │   └── subscription.rs  # 订阅生成
│   ├── dns.rs               # DNS 解析
│   └── network.rs           # 网络工具
├── static/
│   └── index.html           # 首页
├── Dockerfile
├── docker-compose.yml
└── Cargo.toml
```

## 技术栈

- [Tokio](https://tokio.rs/) - 异步运行时
- [Axum](https://github.com/tokio-rs/axum) - Web 框架
- [Tower](https://github.com/tower-rs/tower) - 中间件
- [Reqwest](https://github.com/seanmonstar/reqwest) - HTTP 客户端

## 许可证

MIT License

## 致谢

- 灵感来源: [python-ws](https://github.com/eooce/python-ws)
