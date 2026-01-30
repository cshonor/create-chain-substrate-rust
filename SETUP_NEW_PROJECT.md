# 创建新的 Substrate 项目 - 完整指南

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
2. **Rust 工具链**已配置：
   ```bash
   rustup default stable
   # 编译 WASM 运行时需要 rust-src 组件（stable 和 nightly 都需要）
   rustup component add rust-src --toolchain stable-x86_64-unknown-linux-gnu
   rustup component add rust-src --toolchain nightly
   rustup target add wasm32-unknown-unknown --toolchain nightly
   ```
   
   **注意**：如果编译时遇到错误 "Cannot compile the WASM runtime: no standard library sources found"，说明缺少 `rust-src` 组件，需要为对应的工具链安装。

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

## 注意事项

- 建议将项目移到 WSL 的 Linux 文件系统（`~/substrate`）以获得更好的性能
- 如果使用本地 polkadot-sdk，需要更新 `Cargo.toml` 中的路径配置
- 编译 WASM 运行时需要 `rust-src` 组件，确保已为使用的工具链安装


