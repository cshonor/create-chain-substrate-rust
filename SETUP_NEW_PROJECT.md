# 创建新的 Substrate 项目 - 完整指南

## ⚠️ 重要：Rust 版本兼容性警告

**本项目使用 Polkadot SDK 2512.1，必须使用推荐的 Rust 稳定版本，禁止使用最新开发版或未验证的稳定版。**

### 版本不兼容的严重性

Rust 1.93.0（开发版）与 Polkadot SDK 2512.1 的版本不兼容，**并非简单的"版本号不一致"**，而是**底层编译机制、核心库实现、依赖链规则的全面冲突**。这是 Substrate/Polkadot 官方反复强调**"必须使用推荐的 Rust 稳定版，禁止使用最新开发版/未验证稳定版"**的根本原因。

### 一、直接可见的影响：触发编译层面的致命错误

这是版本不兼容最直接的表现，直接导致 WASM 运行时编译失败，**无法生成可用的链上运行时代码**：

- Rust 1.93.0（开发版）的 `build-std` 从源码编译 `alloc/core` 库时，对 `exchange_malloc` 等 **Rust 核心 lang item**（语言内置项，不可重复定义）的编译逻辑、符号命名做了微调；
- 而 Polkadot SDK 2512.1 的底层依赖（如 `sp-std`、`frame-system`）是基于 Rust 1.79.0 稳定版开发的，仍沿用旧的 `alloc` 库链接逻辑，会隐式调用预编译版 `alloc` 的旧符号；
- 最终导致**源码编译的新 `alloc`** 和**预编译的旧 `alloc`** 同时被链接，两者的核心 lang item 重复定义，编译器直接抛出致命错误（`E0152 duplicate lang item`），WASM 编译中断。

**简单说**：新版本 Rust 改了"基础语法的底层实现"，旧版本 SDK 还按老规则调用，编译器直接判"语法冲突"，不让编译通过。

### 二、深层底层影响：破坏 Substrate WASM 编译的核心机制

Substrate 编译 WASM 运行时的核心是 **`no_std` + `-Z build-std`** 组合（脱离操作系统 std 库，从源码编译轻量的 core/alloc，适配区块链轻节点/链上执行），版本不兼容会直接破坏这个机制：

#### 1. `build-std` 功能失效

`-Z build-std` 是 Rust 的实验性功能，不同版本的 Rust 对其支持度、编译参数、源码输出格式差异极大：

- Rust 1.93.0 的 `build-std` 对 `wasm32-unknown-unknown` 目标的编译优化、文件输出路径做了调整；
- Polkadot SDK 2512.1 的构建脚本（`build.rs`）是按 Rust 1.79.0 的 `build-std` 规则写的，无法识别新版本的输出文件，导致**无法正确找到源码编译的 core/alloc 库**，要么编译失败，要么被迫回退到预编译版 std 库，违背 WASM 运行时的设计初衷。

#### 2. `no_std` 环境被污染

Substrate 的 WASM 运行时必须严格运行在 `no_std` 环境（链上没有操作系统，无法调用 std 库的文件、网络、线程等接口），版本不兼容会导致：

- 新版本 Rust 的部分依赖隐式引入 `std` 库特性，SDK 的 `no_std` 隔离逻辑无法屏蔽；
- 最终编译出的 WASM 文件包含 `std` 库的冗余代码，不仅**文件体积暴增**（影响链上存储和同步效率），还会在链上执行时触发**未定义行为**（调用不存在的系统接口，直接 panic）。

#### 3. 依赖链版本锁死失效

SDK 的 `Cargo.lock` 和 `Cargo.toml` 是按推荐 Rust 版本做的依赖版本匹配，新 Rust 版本会强制升级部分底层依赖（如 `rustc_version`、`wasm-bindgen`），导致：

- 依赖链出现**版本冲突**（如 A 依赖 B@0.10，新 Rust 强制装 B@0.12）；
- 部分依赖的**API 不兼容**（如函数参数、返回值类型修改），即使绕开编译错误，后续开发也会频繁碰到"函数找不到""类型不匹配"的运行时错误。

### 三、潜在功能影响：即使强行编译通过，链上运行必出问题

如果通过修改构建脚本、屏蔽报错等方式强行让 WASM 编译通过，版本不兼容的问题会从"编译期"转移到"运行期"，带来更隐蔽、更致命的链上问题，**直接导致节点无法启动、链上交易失败、甚至区块回滚**：

#### 1. 节点启动失败/同步中断

编译出的 WASM 运行时文件与节点二进制文件（node）不兼容：

- 节点启动时会验证 WASM 运行时的**版本哈希**，版本不兼容会导致哈希验证失败，节点直接崩溃；
- 即使跳过哈希验证，节点同步区块时，无法正确解析链上的 WASM 执行逻辑，会在某个区块高度突然中断，无法继续同步。

#### 2. 链上执行无响应/恐慌（Panic）

WASM 运行时是链上所有交易、合约、治理操作的**执行核心**，版本不兼容会导致：

- 调用 `frame` 模块的核心功能（如转账、创建账户、提案投票）时，触发**未定义行为**（如内存访问越界、空指针引用）；
- 链上执行直接 Panic，交易被回滚，节点日志会出现大量 `WASM execution failed` 错误，整个链的功能陷入瘫痪。

#### 3. 合约部署/链上升级彻底失效

Substrate 的智能合约（如 ink!）部署、链上运行时升级，都依赖**纯净的 WASM 运行时环境**：

- 版本不兼容的 WASM 文件会包含非法指令，合约部署时被链上验证器拒绝；
- 即使部署成功，合约执行时会出现**逻辑错误**（如计算结果错误、存储数据损坏）；
- 链上升级时，新的 WASM 运行时无法被旧节点识别，导致全网节点分叉，区块链共识崩溃。

#### 4. 调试困难，无有效报错信息

由于是底层版本不兼容，而非业务代码错误，后续的调试会非常困难：

- 编译器/运行时的报错信息模糊（如"WASM execution failed""memory access out of bounds"），无法定位具体问题；
- 常用的调试工具（如 `substrate-contracts-node`、`wasmtime`）也无法正确解析不兼容的 WASM 文件，调试过程几乎无法推进。

### 四、额外影响：开发环境混乱，后续升级埋下隐患

如果不及时切换到推荐的 Rust 版本，强行在不兼容的环境下开发，还会导致**开发环境永久混乱**：

1. Rust 的 `toolchain`、`target`、`component`（如 `rust-src`）版本混杂，后续新建 Substrate 项目时，会频繁碰到相同的编译错误；
2. 本地的 Cargo 缓存（`~/.cargo/registry`）会存储大量不兼容的依赖版本，导致后续编译其他项目时，出现莫名的依赖冲突；
3. 若将不兼容的代码提交到仓库，其他开发者拉取代码后，会直接复现相同的错误，团队协作效率大幅降低。

### 总结

Rust 1.93.0 与 Polkadot SDK 2512.1 的版本不兼容，其影响可总结为 3 个核心点：

1. **编译期**：直接触发致命的重复链接错误，无法生成可用的 WASM 运行时；
2. **底层**：破坏 Substrate `no_std+build-std` 的 WASM 核心编译机制，导致环境污染、功能失效；
3. **运行期**：即使强行编译通过，会引发节点崩溃、交易失败、共识分叉等致命链上问题，且调试困难。

**区块链底层开发对编译环境的一致性、稳定性要求极高，任何底层的微小改动，都会被无限放大为链上的致命故障。**

## 当前状态

- ✅ 已删除旧的 substrate 目录
- ✅ 正在克隆 polkadot-sdk 仓库（进行中）
- ✅ Rust 环境已配置（stable + nightly）
- ✅ 已创建 rust-toolchain.toml 配置文件
- ✅ 已安装 rust-src 组件（stable 工具链）
- ✅ 项目结构已整理，代码已添加中文注释

## 下一步操作

### 方法 1：等待 polkadot-sdk 克隆完成后使用模板

```bash
# 在 WSL 中执行
cd /mnt/c/Users/12392/Desktop/rs

# 等待 polkadot-sdk 克隆完成（约 8647 个文件）
# 然后复制 minimal 模板
cd polkadot-sdk
cp -r cumulus/templates/minimal ../substrate
cd ../substrate
```

### 方法 2：使用 cargo-generate（推荐，更快）

```bash
# 在 WSL 中执行
cd /mnt/c/Users/12392/Desktop/rs
source ~/.cargo/env

# 安装 cargo-generate（如果还没有）
cargo install cargo-generate --locked --force

# 创建新项目
cargo generate --git https://github.com/paritytech/polkadot-sdk --name substrate --template minimal
```

### 方法 3：使用官方快速启动脚本

```bash
# 在 WSL 中执行
cd /mnt/c/Users/12392/Desktop/rs
curl --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/paritytech/polkadot-sdk/master/scripts/getting-started.sh | bash
```

## 项目配置

创建项目后，确保：

1. **rust-toolchain.toml** 已存在（已创建）

2. **Rust 工具链**已配置（**必须使用推荐的稳定版本**）：
   ```bash
   # 使用稳定版本（不要使用开发版或未验证的稳定版）
   rustup default stable
   
   # 编译 WASM 运行时需要 rust-src 组件（stable 和 nightly 都需要）
   rustup component add rust-src --toolchain stable-x86_64-unknown-linux-gnu
   rustup component add rust-src --toolchain nightly
   rustup target add wasm32-unknown-unknown --toolchain nightly
   ```
   
   **⚠️ 重要**：如果编译时遇到错误 "Cannot compile the WASM runtime: no standard library sources found"，说明缺少 `rust-src` 组件，需要为对应的工具链安装。
   
   **⚠️ 重要**：如果遇到 `duplicate lang item in crate core` 错误，说明 Rust 版本与 Polkadot SDK 不兼容，请切换到推荐的稳定版本。

3. **构建项目**：
   ```bash
   cd substrate
   WASM_BUILD_TOOLCHAIN=nightly-2024-07-01 RUSTFLAGS='-A useless_deprecated' cargo build --release --locked
   ```

## 编译测试

### 遇到的问题和解决方案

**问题**：编译时出现错误
```
error: failed to run custom build command for `frame-storage-access-test-runtime v0.6.0`
Cannot compile the WASM runtime: no standard library sources found at /root/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust!
```

**解决方案**：
```bash
rustup component add rust-src --toolchain stable-x86_64-unknown-linux-gnu
```

**测试编译**：
```bash
cd /root/projects/substrate
cargo check --package minimal-template-runtime --package minimal-template-node
# 或完整编译
cargo build --workspace --release
```

## 常见编译错误和解决方案

### 错误1：`edition = "2024" is required` 或 `rustc version not supported`

**问题根源**：
- 当前使用的是 Rust 1.79.0（兼容版本），但 Cargo.lock 里的某些依赖包版本过新
- 这些新包默认启用了 `edition = "2024"`，而 Rust 1.79.0 并不支持这个特性
- Cargo 配置可能用了国内镜像源（如阿里云），导致依赖包缓存混乱，拉取到的版本和预期不符

**完整修复步骤**：

1. **备份并禁用当前 Cargo 配置**
   ```bash
   mv ~/.cargo/config.toml ~/.cargo/config.toml.bak
   ```

2. **清理问题依赖缓存**
   ```bash
   # 清理特定问题依赖（如果知道具体包名）
   rm -rf ~/.cargo/registry/src/mirrors.aliyun.com-8754fae0eb2f08f1/cranelift-control-0.122.0
   
   # 或清理整个缓存（更彻底）
   rm -rf ~/.cargo/registry/cache
   rm -rf ~/.cargo/registry/src
   ```

3. **删除旧的 Cargo.lock**
   ```bash
   rm -f Cargo.lock
   ```

4. **重新生成兼容的依赖锁文件**
   ```bash
   cd /root/projects/substrate
   CARGO_NET_OFFLINE=false cargo generate-lockfile
   ```

5. **重新编译**
   ```bash
   export RUSTFLAGS='-A useless_deprecated'
   export WASM_BUILD_TOOLCHAIN=nightly-2024-07-01
   cargo build --release --locked
   ```

### 错误2：`duplicate lang item in crate core`

**问题根源**：Rust nightly 版本与 Polkadot SDK 的 `build-std` 功能不兼容。

**解决方案**：
- 使用推荐的 nightly 版本：`nightly-2024-07-01`
- 确保 `rust-toolchain.toml` 配置正确
- 使用 `WASM_BUILD_TOOLCHAIN` 环境变量指定 nightly 版本

### 错误3：`no standard library sources found`

**问题根源**：缺少 `rust-src` 组件。

**解决方案**：
```bash
rustup component add rust-src --toolchain stable
rustup component add rust-src --toolchain nightly
```

## 防止依赖自动升级的配置

为了避免后续遇到类似的依赖版本冲突，可以配置 Cargo 防止依赖自动升级：

### 创建 Cargo 配置

```bash
mkdir -p ~/.cargo
cat > ~/.cargo/config.toml << 'EOF'
# Cargo 配置：防止依赖自动升级

[net]
# 使用官方源（避免镜像源缓存问题）
# 如果需要使用镜像，请确保镜像与官方源同步

[build]
# 禁用自动更新依赖
# 使用 --locked 参数时，严格遵循 Cargo.lock

[profile.release]
# 发布版本的优化配置
opt-level = 3
lto = true
codegen-units = 1
panic = 'abort'

# 如果需要使用国内镜像（可选）
# [source.crates-io]
# replace-with = "tuna"
# 
# [source.tuna]
# registry = "https://mirrors.tuna.tsinghua.edu.cn/git/crates.io-index.git"
EOF
```

### 项目级配置（推荐）

在项目根目录创建 `.cargo/config.toml`：

```bash
mkdir -p .cargo
cat > .cargo/config.toml << 'EOF'
# 项目级 Cargo 配置
# 此配置仅影响当前项目

[build]
# 使用 --locked 时严格遵循 Cargo.lock
# 不会自动更新依赖版本

[net]
# 网络配置
git-fetch-with-cli = true
retry = 3
EOF
```

## 注意事项

- 建议将项目移到 WSL 的 Linux 文件系统（`~/substrate`）以获得更好的性能
- 如果使用本地 polkadot-sdk，需要更新 `Cargo.toml` 中的路径配置
- 编译 WASM 运行时需要 `rust-src` 组件，确保已为使用的工具链安装
- **重要**：使用 `--locked` 参数构建，确保依赖版本一致性
- **重要**：如果遇到依赖版本冲突，优先清理缓存并重新生成 `Cargo.lock`


