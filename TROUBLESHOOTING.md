# Substrate 项目故障排查指南

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
```bash
# 使用推荐的 nightly 版本
export WASM_BUILD_TOOLCHAIN=nightly-2024-07-01

# 确保 rust-toolchain.toml 配置正确
# 检查并安装正确的 nightly 版本
rustup toolchain install nightly-2024-07-01
rustup target add wasm32-unknown-unknown --toolchain nightly-2024-07-01

# 重新编译
cargo build --release --locked
```

### 错误3：`no standard library sources found`

**问题根源**：缺少 `rust-src` 组件。

**解决方案**：
```bash
# 为 stable 工具链安装 rust-src
rustup component add rust-src --toolchain stable

# 为 nightly 工具链安装 rust-src
rustup component add rust-src --toolchain nightly

# 验证安装
rustup component list --toolchain stable | grep rust-src
rustup component list --toolchain nightly | grep rust-src
```

### 错误4：依赖下载失败（网络问题）

**问题根源**：网络连接不稳定或镜像源配置问题。

**解决方案**：

**方案A：使用官方源（推荐）**
```bash
# 备份当前配置
mv ~/.cargo/config.toml ~/.cargo/config.toml.bak

# 使用官方源重新下载
cargo build --release
```

**方案B：配置国内镜像（如果必须）**
```bash
mkdir -p ~/.cargo
cat > ~/.cargo/config.toml << 'EOF'
[source.crates-io]
replace-with = "tuna"

[source.tuna]
registry = "https://mirrors.tuna.tsinghua.edu.cn/git/crates.io-index.git"

[net]
git-fetch-with-cli = true
retry = 3
EOF
```

### 错误5：内存不足（OOM）

**问题根源**：编译大型 Substrate 项目需要大量内存。

**解决方案**：
```bash
# 减少并行编译任务数
export CARGO_BUILD_JOBS=2

# 或使用更少的核心
export CARGO_BUILD_JOBS=4

# 重新编译
cargo build --release --locked
```

## 防止依赖自动升级

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

### 全局配置

```bash
mkdir -p ~/.cargo
cat > ~/.cargo/config.toml << 'EOF'
# 全局 Cargo 配置

[net]
# 使用官方源（避免镜像源缓存问题）
git-fetch-with-cli = true
retry = 3

[build]
# 禁用自动更新依赖
# 使用 --locked 参数时，严格遵循 Cargo.lock
EOF
```

## 构建优化建议

### 1. 使用正确的 Rust 版本

```bash
# 检查当前 Rust 版本
rustc --version

# 使用推荐的稳定版本
rustup default stable

# 安装推荐的 nightly 版本
rustup toolchain install nightly-2024-07-01
```

### 2. 配置构建参数

```bash
# 设置并行任务数（根据 CPU 核心数调整）
export CARGO_BUILD_JOBS=6

# 设置 Rust 编译标志
export RUSTFLAGS='-A useless_deprecated'

# 设置 Wasm 构建工具链
export WASM_BUILD_TOOLCHAIN=nightly-2024-07-01
```

### 3. 使用构建缓存

```bash
# Cargo 会自动使用缓存，无需额外配置
# 首次构建后，后续构建会快很多

# 清理缓存（如果遇到问题）
cargo clean

# 只清理特定包的缓存
rm -rf target/release/deps/包名*
```

## 快速修复脚本

创建 `fix_build.sh` 脚本：

```bash
#!/bin/bash
# 一键修复常见编译问题

set -e

echo "=========================================="
echo "开始修复编译问题..."
echo "=========================================="

# 1. 备份 Cargo 配置
if [ -f ~/.cargo/config.toml ]; then
    echo "[1/5] 备份 Cargo 配置..."
    mv ~/.cargo/config.toml ~/.cargo/config.toml.bak
fi

# 2. 清理缓存
echo "[2/5] 清理依赖缓存..."
rm -rf ~/.cargo/registry/cache
rm -rf ~/.cargo/registry/src

# 3. 删除 Cargo.lock
echo "[3/5] 删除旧的 Cargo.lock..."
rm -f Cargo.lock

# 4. 重新生成锁文件
echo "[4/5] 重新生成 Cargo.lock..."
CARGO_NET_OFFLINE=false cargo generate-lockfile

# 5. 重新编译
echo "[5/5] 重新编译..."
export RUSTFLAGS='-A useless_deprecated'
export WASM_BUILD_TOOLCHAIN=nightly-2024-07-01
cargo build --release --locked

echo "=========================================="
echo "修复完成！"
echo "=========================================="
```

使用方法：
```bash
chmod +x fix_build.sh
./fix_build.sh
```

## 验证修复

修复后，验证编译是否成功：

```bash
# 检查二进制文件
ls -lh target/release/minimal-template-node

# 测试运行
./target/release/minimal-template-node --version

# 或启动节点测试
./target/release/minimal-template-node --dev --tmp
```

