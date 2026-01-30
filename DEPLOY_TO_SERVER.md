# 将 Substrate 项目部署到阿里云服务器并构建

## 服务器信息
- **公网IP**: 8.148.205.248
- **配置**: 8核32G
- **系统**: Ubuntu 22.04

## 方案一：通过 Git 克隆（推荐）

### 步骤1：在服务器上初始化环境

```bash
# 1. 更新系统
sudo apt update -y
sudo apt upgrade -y

# 2. 安装基础工具
sudo apt install -y git curl wget build-essential

# 3. 安装 Rust（如果还没有）
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source $HOME/.cargo/env

# 4. 配置 Rust 工具链
rustup default stable
rustup component add rustfmt clippy
rustup target add wasm32-unknown-unknown
rustup toolchain install nightly
rustup target add wasm32-unknown-unknown --toolchain nightly

# 5. 安装 Substrate 编译依赖
sudo apt install -y \
    clang \
    libclang-dev \
    libssl-dev \
    pkg-config \
    libudev-dev \
    protobuf-compiler \
    cmake \
    libsqlite3-dev \
    libzstd-dev
```

### 步骤2：克隆项目到服务器

```bash
# 创建项目目录
mkdir -p ~/projects
cd ~/projects

# 方式1：克隆你的 GitHub 仓库（推荐）
git clone https://github.com/cshonor/create-chain-substrate-rust.git substrate
cd substrate

# 方式2：如果仓库是私有的，需要配置 SSH 密钥或使用 HTTPS + token
# git clone git@github.com:cshonor/create-chain-substrate-rust.git substrate
```

### 步骤3：构建项目

```bash
# 进入项目目录
cd ~/projects/substrate

# 优化构建参数（8核32G配置，使用6个并行任务）
export CARGO_BUILD_JOBS=6
export RUSTFLAGS='-A useless_deprecated'
export WASM_BUILD_TOOLCHAIN=nightly-2024-07-01

# 构建发布版本（首次构建需要较长时间，约30-60分钟）
cargo build --release --locked

# 构建完成后，可执行文件位于：
# target/release/minimal-template-node
```

## 方案二：通过 SCP 上传项目文件

### 步骤1：在本地打包项目

在本地 Windows PowerShell 执行：

```powershell
# 进入项目目录
cd C:\Users\12392\Desktop\rs\substrate

# 创建压缩包（排除 target 目录，因为可以在服务器上重新构建）
tar -czf substrate-project.tar.gz --exclude='target' --exclude='.git' .
```

### 步骤2：上传到服务器

```powershell
# 上传压缩包到服务器
scp substrate-project.tar.gz root@8.148.205.248:~/

# SSH 连接到服务器
ssh root@8.148.205.248
```

### 步骤3：在服务器上解压和构建

```bash
# 创建项目目录
mkdir -p ~/projects
cd ~/projects

# 解压项目
tar -xzf ~/substrate-project.tar.gz -C ~/projects/
cd ~/projects/substrate

# 初始化 Git（如果需要）
git init
git remote add origin https://github.com/cshonor/create-chain-substrate-rust.git

# 构建项目（参考方案一的步骤3）
export CARGO_BUILD_JOBS=6
export RUSTFLAGS='-A useless_deprecated'
export WASM_BUILD_TOOLCHAIN=nightly-2024-07-01
cargo build --release --locked
```

## 方案三：使用 Docker 构建（推荐用于生产环境）

### 步骤1：在服务器上安装 Docker

```bash
# 安装 Docker
sudo apt update -y
sudo apt install -y ca-certificates gnupg lsb-release
curl -fsSL https://download.docker.com/linux/ubuntu/gpg | sudo gpg --dearmor -o /etc/apt/trusted.gpg.d/docker.gpg
echo "deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/trusted.gpg.d/docker.gpg] https://download.docker.com/linux/ubuntu $(lsb_release -cs) stable" | sudo tee /etc/apt/sources.list.d/docker.list > /dev/null
sudo apt update -y
sudo apt install -y docker-ce docker-ce-cli containerd.io
sudo systemctl enable --now docker
```

### 步骤2：使用 Dockerfile 构建

```bash
# 克隆项目
cd ~/projects
git clone https://github.com/cshonor/create-chain-substrate-rust.git substrate
cd substrate

# 使用 Docker 构建（如果项目有 Dockerfile）
docker build -t substrate-node:latest .

# 或使用官方 Substrate 构建镜像
docker run --rm -v $(pwd):/build -w /build paritytech/substrate-build-env:latest cargo build --release
```

## 构建优化建议（8核32G配置）

### 1. 并行构建任务数

```bash
# 使用6个并行任务（留2个核心给系统）
export CARGO_BUILD_JOBS=6

# 或使用所有核心（8个）
export CARGO_BUILD_JOBS=8
```

### 2. 内存优化

```bash
# 32G内存足够，无需特殊配置
# 但如果遇到内存不足，可以减少并行任务数
export CARGO_BUILD_JOBS=4
```

### 3. 构建缓存

```bash
# 使用本地缓存加速后续构建
# Cargo 会自动使用 ~/.cargo/registry 和项目 target 目录的缓存
```

### 4. 增量编译

```bash
# Cargo 默认启用增量编译，无需额外配置
# 首次构建后，修改代码后的重新构建会快很多
```

## 完整构建脚本

创建 `build.sh` 脚本：

```bash
#!/bin/bash
# Substrate 项目构建脚本

set -e

echo "=========================================="
echo "开始构建 Substrate 项目..."
echo "=========================================="

# 检查 Rust 环境
if ! command -v rustc &> /dev/null; then
    echo "错误: 未找到 Rust，请先安装 Rust"
    exit 1
fi

# 设置构建参数
export CARGO_BUILD_JOBS=6
export RUSTFLAGS='-A useless_deprecated'
export WASM_BUILD_TOOLCHAIN=nightly-2024-07-01

# 显示环境信息
echo "Rust 版本: $(rustc --version)"
echo "Cargo 版本: $(cargo --version)"
echo "CPU 核心数: $(nproc)"
echo "可用内存: $(free -h | grep Mem | awk '{print $7}')"
echo "并行构建任务数: $CARGO_BUILD_JOBS"
echo ""

# 清理之前的构建（可选）
# cargo clean

# 开始构建
echo "开始构建发布版本..."
time cargo build --release --locked

# 检查构建结果
if [ -f "target/release/minimal-template-node" ]; then
    echo ""
    echo "=========================================="
    echo "构建成功！"
    echo "=========================================="
    echo "可执行文件位置: $(pwd)/target/release/minimal-template-node"
    echo "文件大小: $(du -h target/release/minimal-template-node | cut -f1)"
    echo ""
    echo "运行节点:"
    echo "  ./target/release/minimal-template-node --dev --tmp"
else
    echo "错误: 构建失败，未找到可执行文件"
    exit 1
fi
```

使用脚本：

```bash
# 给脚本添加执行权限
chmod +x build.sh

# 执行构建
./build.sh
```

## 运行节点

构建完成后，运行节点：

```bash
# 开发模式（临时数据，重启后清空）
./target/release/minimal-template-node --dev --tmp

# 生产模式（持久化数据）
./target/release/minimal-template-node \
  --chain dev \
  --base-path ~/substrate-data \
  --name MySubstrateNode \
  --rpc-external \
  --ws-external \
  --rpc-cors all \
  --port 30333 \
  --ws-port 9944
```

## 后台运行（使用 systemd）

创建 systemd 服务文件 `/etc/systemd/system/substrate-node.service`：

```ini
[Unit]
Description=Substrate Node
After=network.target

[Service]
Type=simple
User=root
WorkingDirectory=/root/projects/substrate
ExecStart=/root/projects/substrate/target/release/minimal-template-node \
  --chain dev \
  --base-path /root/substrate-data \
  --name MySubstrateNode \
  --rpc-external \
  --ws-external \
  --rpc-cors all
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
```

启动服务：

```bash
# 重新加载 systemd
sudo systemctl daemon-reload

# 启动服务
sudo systemctl start substrate-node

# 设置开机自启
sudo systemctl enable substrate-node

# 查看日志
sudo journalctl -u substrate-node -f
```

## 监控和日志

```bash
# 查看节点进程
ps aux | grep minimal-template-node

# 查看资源使用
htop

# 查看节点日志（如果使用 systemd）
sudo journalctl -u substrate-node -f --lines=100

# 查看节点状态（如果启用了 Prometheus）
curl http://localhost:9615/metrics
```

## 常见问题

### Q: 构建时间太长？
- **首次构建**：正常，需要30-60分钟
- **后续构建**：使用增量编译，通常只需几分钟
- **优化**：使用 `CARGO_BUILD_JOBS=6` 充分利用多核

### Q: 内存不足？
- 32G内存通常足够
- 如果遇到问题，减少 `CARGO_BUILD_JOBS` 到 4

### Q: 网络问题导致依赖下载失败？
```bash
# 配置 Rust 镜像（如果在中国）
mkdir -p ~/.cargo
cat > ~/.cargo/config.toml << EOF
[source.crates-io]
replace-with = "tuna"

[source.tuna]
registry = "https://mirrors.tuna.tsinghua.edu.cn/git/crates.io-index.git"

[net]
git-fetch-with-cli = true
retry = 3
EOF
```

### Q: Wasm 编译失败？
```bash
# 确保安装了 nightly 工具链
rustup toolchain install nightly
rustup target add wasm32-unknown-unknown --toolchain nightly

# 设置环境变量
export WASM_BUILD_TOOLCHAIN=nightly-2024-07-01
```

## 快速开始命令（一键执行）

```bash
# 在服务器上执行以下命令序列：

# 1. 安装 Rust（如果还没有）
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source $HOME/.cargo/env

# 2. 配置 Rust
rustup default stable
rustup component add rustfmt clippy
rustup target add wasm32-unknown-unknown
rustup toolchain install nightly
rustup target add wasm32-unknown-unknown --toolchain nightly

# 3. 安装依赖
sudo apt update -y
sudo apt install -y git clang libclang-dev libssl-dev pkg-config libudev-dev protobuf-compiler cmake libsqlite3-dev libzstd-dev

# 4. 克隆项目
mkdir -p ~/projects && cd ~/projects
git clone https://github.com/cshonor/create-chain-substrate-rust.git substrate
cd substrate

# 5. 构建项目
export CARGO_BUILD_JOBS=6
export RUSTFLAGS='-A useless_deprecated'
export WASM_BUILD_TOOLCHAIN=nightly-2024-07-01
cargo build --release --locked

# 6. 运行节点
./target/release/minimal-template-node --dev --tmp
```

---

**推荐方案**：使用**方案一（Git 克隆）**，最简单且便于后续更新代码。

