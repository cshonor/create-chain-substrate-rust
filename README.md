# Substrate 区块链开发项目

基于 Polkadot SDK 的 Substrate 区块链节点模板项目，用于快速开始区块链开发。

## 📋 项目简介

本项目包含一个最小化的 Substrate 节点模板（`node-template`），基于 Polkadot SDK 的 `templates/minimal`，提供了一个完整的、可运行的区块链节点实现。

### 核心组件

根据 Substrate 框架架构，本项目包含以下核心组件：

#### 1. 核心客户端 (Core Client)
- **P2P 网络**: 使用 Rust 的 libp2p 网络栈
- **共识**: 共识引擎提供的逻辑
- **持久化存储**: 简单高效的键值存储
- **交易请求**: 将被包含在区块中的数据，通常称为 "extrinsics"
- **RPC 远程过程调用 API**: 处理 HTTP 和 WebSocket 请求
- **遥测**: 使用 Prometheus 监控节点性能

#### 2. 运行时 (Runtime)
- 负责链上事件、交易有效性和状态变更的核心组件
- 编译为 Wasm (WebAssembly) 字节码
- **特性**:
  - ✅ 支持无分叉升级
  - ✅ 多平台兼容性
  - ✅ 运行时有效性检查
  - ✅ 中继链共识机制的验证证明

## 🗂️ 项目结构

```
substrate/
├── node-template/          # Substrate 节点模板
│   ├── node/               # 节点二进制（CLI、网络、共识装配）
│   ├── runtime/            # 运行时（FRAME pallets、executive、metadata）
│   ├── pallets/            # 自定义本地 Pallet
│   │   └── template/       # 示例模板 Pallet
│   ├── Cargo.toml          # 工作区配置
│   └── README.md           # 详细使用说明
├── polkadot-sdk/           # Polkadot SDK 源码（如需要）
└── README.md               # 本文件
```

## 🚀 快速开始

### 环境要求

- **Rust**: nightly 工具链
- **系统依赖**: 
  - Windows: 建议使用 WSL (Windows Subsystem for Linux)
  - Linux: `build-essential`, `clang`, `libssl-dev`, `protobuf-compiler`, `libclang-dev`

### 安装 Rust 环境

```bash
# 安装 Rust (如果还没有)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 安装 nightly 工具链
rustup toolchain add nightly
rustup default nightly

# 添加必要的组件和目标
rustup component add rust-src
rustup target add wasm32-unknown-unknown
```

### 构建项目

进入 `node-template` 目录：

```bash
cd node-template
cargo build --release
```

首次构建会下载并编译大量依赖，可能需要较长时间。

### 运行节点

以开发模式启动节点（数据临时存储，便于调试）：

```bash
./target/release/node-template --dev
```

或者使用手动共识模式：

```bash
./target/release/node-template --tmp --consensus manual-seal-3000
```

### 连接前端

节点启动后，可以通过以下方式连接：

- **RPC**: http://127.0.0.1:9933
- **WebSocket**: ws://127.0.0.1:9944

使用 [Polkadot.js Apps](https://polkadot.js.org/apps/#/explorer?rpc=ws://localhost:9944) 连接到本地节点进行交互。

## 📚 详细文档

更多详细信息请查看 [`node-template/README.md`](./node-template/README.md)，包含：

- 完整的安装和配置说明
- Windows/WSL 构建指南
- 常见问题解答
- 项目结构详解
- 开发最佳实践

## 💻 代码位置指南

在 Substrate 项目中，不同功能的代码位于不同的位置：

### 📍 快速参考表

| 要做什么 | 文件位置 | 说明 |
|---------|---------|------|
| **编写业务逻辑** | `pallets/template/src/lib.rs` | ⭐ 最常用！编写链上功能模块 |
| **配置运行时** | `runtime/src/lib.rs` | 配置哪些 Pallet 被使用 |
| **修改链规格** | `node/src/chain_spec.rs` | 设置创世区块、初始账户 |
| **自定义 RPC** | `node/src/rpc.rs` | 添加自定义 RPC 方法 |
| **修改依赖** | `runtime/Cargo.toml` | 添加新的 Pallet 依赖 |
| **工作区配置** | `Cargo.toml` | 管理整个项目的依赖 |

### 详细说明

### 1. **自定义业务逻辑 - Pallet** ⭐ 最常用

**位置**: `node-template/pallets/template/src/lib.rs`

这是编写链上业务逻辑的主要位置。每个 Pallet 代表一个功能模块（如代币、治理、NFT 等）。

**示例代码结构**:
```rust
#[frame::pallet]
pub mod pallet {
    // 配置 trait
    #[pallet::config]
    pub trait Config: frame_system::Config {}
    
    // 存储项
    #[pallet::storage]
    pub type Value<T> = StorageValue<Value = u32>;
    
    // 可调用函数（用户可调用的交易）
    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        pub fn do_something(origin: OriginFor<T>) -> DispatchResult {
            // 你的业务逻辑
            Ok(())
        }
    }
}
```

**创建新 Pallet**:
1. 在 `pallets/` 目录下创建新文件夹（如 `pallets/my-pallet/`）
2. 创建 `Cargo.toml` 和 `src/lib.rs`
3. 在 `runtime/Cargo.toml` 中添加依赖
4. 在 `runtime/src/lib.rs` 中引入和配置

### 2. **运行时配置 - Runtime**

**位置**: `node-template/runtime/src/lib.rs`

这里配置哪些 Pallet 被包含在运行时中，以及它们的参数设置。

**主要工作**:
- 引入自定义 Pallet
- 配置 Pallet 参数
- 定义运行时 API
- 设置创世配置

**示例**:
```rust
// 引入你的 pallet
pub use pallet_template;

// 在 construct_runtime! 宏中配置
construct_runtime! {
    pub enum Runtime {
        // ... 其他 pallets
        TemplatePallet: pallet_template,
    }
}
```

### 3. **节点配置 - Node**

**位置**: `node-template/node/src/`

主要用于节点层面的配置，通常不需要修改：

- **`lib.rs`**: 节点服务组装
- **`chain_spec.rs`**: 链规格配置（创世区块、初始账户等）
- **`cli.rs`**: 命令行参数定义
- **`rpc.rs`**: RPC API 扩展（如需要自定义 RPC）

**何时修改**:
- 需要自定义链规格（初始账户、余额等）
- 需要添加自定义 RPC 方法
- 需要修改节点启动参数

### 4. **项目配置文件**

- **`node-template/Cargo.toml`**: 工作区配置，定义所有子项目
- **`node-template/runtime/Cargo.toml`**: 运行时依赖
- **`node-template/pallets/*/Cargo.toml`**: 各个 Pallet 的依赖

## 🛠️ 开发指南

### 添加自定义 Pallet

1. 在 `pallets/` 目录下创建新的 pallet 文件夹
2. 创建 `Cargo.toml` 和 `src/lib.rs` 文件
3. 在 `runtime/Cargo.toml` 中添加依赖
4. 在 `runtime/src/lib.rs` 中引入和配置 pallet

### 运行测试

```bash
cargo test
```

### 代码检查

```bash
cargo check
```

## 📖 学习资源

- [Substrate 官方文档](https://docs.substrate.io/)
- [Polkadot SDK 文档](https://github.com/paritytech/polkadot-sdk#-documentation)
- [Polkadot 文档](https://docs.polkadot.com/)
- [Substrate StackExchange](https://substrate.stackexchange.com/)

## 🤝 贡献

本项目基于 [Polkadot SDK](https://github.com/paritytech/polkadot-sdk) 的模板。如有问题或建议，请参考：

- [Polkadot SDK Issues](https://github.com/paritytech/polkadot-sdk/issues)
- [Polkadot Discord](https://polkadot-discord.w3f.tools/)
- [Substrate Telegram](https://t.me/substratedevs)

## 📄 许可证

本项目遵循相应的开源许可证。请查看各子目录中的 LICENSE 文件了解详情。

---

**注意**: 这是一个最小化模板，主要用于学习和实验。在生产环境使用前，请确保添加适当的共识机制、安全审计和性能优化。

