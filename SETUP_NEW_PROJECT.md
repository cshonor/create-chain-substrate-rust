# 创建新的 Substrate 项目 - 完整指南

## 当前状态

- ✅ 已删除旧的 substrate 目录
- ✅ 正在克隆 polkadot-sdk 仓库（进行中）
- ✅ Rust 环境已配置（stable + nightly）
- ✅ 已创建 rust-toolchain.toml 配置文件

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
   rustup component add rust-src --toolchain nightly
   rustup target add wasm32-unknown-unknown --toolchain nightly
   ```

3. **构建项目**：
   ```bash
   cd substrate
   WASM_BUILD_TOOLCHAIN=nightly-2024-07-01 RUSTFLAGS='-A useless_deprecated' cargo build --release --locked
   ```

## 注意事项

- 建议将项目移到 WSL 的 Linux 文件系统（`~/substrate`）以获得更好的性能
- 如果使用本地 polkadot-sdk，需要更新 `Cargo.toml` 中的路径配置


