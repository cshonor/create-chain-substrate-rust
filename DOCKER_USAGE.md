# Docker 使用指南

## 📦 Docker 是什么？

Docker 镜像就像一个"打包好的开发盒子"，里面装好了你项目需要的所有环境（比如特定版本的 Rust、Substrate 依赖、编译工具等等）。不管你在什么电脑上，只要有 Docker，打开这个"盒子"就能直接运行你的代码，不用再重新配置环境。

## 🚀 快速开始

### 前置要求

1. **安装 Docker Desktop**
   - Windows: [下载 Docker Desktop](https://www.docker.com/products/docker-desktop/)
   - Mac: [下载 Docker Desktop](https://www.docker.com/products/docker-desktop/)
   - Linux: `sudo apt-get install docker.io docker-compose`

2. **配置 Docker 资源**（推荐）
   - 打开 Docker Desktop 设置
   - 分配至少 **8GB 内存** 和 **2 核 CPU**
   - 启用 WSL2 后端（Windows）

### 方式 1：使用 Docker Compose（最简单 ⭐）

```bash
# 构建并启动节点
docker-compose up -d

# 查看日志
docker-compose logs -f

# 停止节点
docker-compose down

# 停止并删除数据卷（清理链数据）
docker-compose down -v
```

### 方式 2：使用 Docker 命令

#### 构建镜像

```bash
# 构建镜像（首次构建需要 10-30 分钟，取决于网络速度）
docker build -t minimal-template-node:latest .

# 查看构建的镜像
docker images | grep minimal-template-node
```

#### 运行容器

```bash
# 启动开发模式节点
docker run -d \
  --name substrate-node \
  -p 30333:30333 \
  -p 9933:9933 \
  -p 9944:9944 \
  -p 9615:9615 \
  -v substrate-data:/data \
  minimal-template-node:latest \
  --dev --rpc-external --ws-external --rpc-cors=all

# 查看日志
docker logs -f substrate-node

# 停止容器
docker stop substrate-node

# 删除容器
docker rm substrate-node
```

## 🔧 常用操作

### 查看节点状态

```bash
# 查看运行中的容器
docker ps

# 查看节点日志
docker logs -f substrate-node

# 进入容器内部（调试用）
docker exec -it substrate-node /bin/sh
```

### 连接到节点

节点启动后，可以通过以下方式连接：

- **Polkadot-JS Apps**: http://localhost:9944
- **RPC HTTP**: http://localhost:9933
- **WebSocket**: ws://localhost:9944
- **Prometheus 指标**: http://localhost:9615/metrics

### 使用自定义命令

```bash
# 查看帮助
docker run --rm minimal-template-node:latest --help

# 启动生产模式节点
docker run -d \
  --name substrate-node \
  -p 30333:30333 \
  -p 9933:9933 \
  -p 9944:9944 \
  -v substrate-data:/data \
  minimal-template-node:latest \
  --chain=local \
  --alice \
  --rpc-external \
  --ws-external

# 导出链规范
docker run --rm minimal-template-node:latest build-spec --chain=dev > chain-spec.json
```

## 🛠️ 开发模式

### 挂载本地代码（热重载）

如果你想在 Docker 中直接修改代码并重新编译：

```bash
# 方式 1：使用 docker-compose.yml（取消注释 volumes 部分）
# volumes:
#   - ./:/substrate

# 方式 2：使用 docker run
docker run -it --rm \
  -v $(pwd):/substrate \
  -w /substrate \
  docker.io/paritytech/ci-unified:latest \
  bash

# 进入容器后
cd /substrate
cargo build --release
./target/release/minimal-template-node --dev
```

### 在容器中编译

```bash
# 启动一个临时容器用于编译
docker run -it --rm \
  -v $(pwd):/substrate \
  -w /substrate \
  docker.io/paritytech/ci-unified:latest \
  cargo build --release

# 编译后的二进制文件在本地 target/release/ 目录
```

## 📤 从 GitHub 克隆并运行

如果你从 GitHub 克隆了这个项目：

```bash
# 1. 克隆项目
git clone <your-github-repo-url>
cd substrate

# 2. 构建 Docker 镜像
docker build -t my-substrate-project .

# 3. 运行节点
docker run -d \
  --name my-substrate-node \
  -p 9944:9944 \
  my-substrate-project

# 4. 查看日志
docker logs -f my-substrate-node
```

## 🐛 故障排查

### 问题 1：构建失败 - 内存不足

**错误信息**：
```
error: failed to run custom build command for `...`
```

**解决方案**：
1. 增加 Docker 内存分配（Docker Desktop 设置 → Resources → Memory → 至少 8GB）
2. 减少并行编译任务：在 Dockerfile 中设置 `ENV CARGO_BUILD_JOBS=2`

### 问题 2：端口已被占用

**错误信息**：
```
Error: bind: address already in use
```

**解决方案**：
```bash
# 查看占用端口的进程
# Windows
netstat -ano | findstr :9944

# Linux/Mac
lsof -i :9944

# 修改 docker-compose.yml 中的端口映射
# 例如：- "9945:9944"  # 使用 9945 作为外部端口
```

### 问题 3：数据卷权限问题

**错误信息**：
```
Permission denied: /data
```

**解决方案**：
```bash
# 删除旧的数据卷并重新创建
docker-compose down -v
docker-compose up -d
```

### 问题 4：网络连接问题（拉取镜像失败）

**解决方案**：
```bash
# 配置 Docker 镜像加速（国内用户）
# 编辑 Docker Desktop 设置 → Docker Engine
# 添加：
{
  "registry-mirrors": [
    "https://docker.mirrors.ustc.edu.cn",
    "https://hub-mirror.c.163.com"
  ]
}
```

## 📊 资源使用

### 典型资源占用

- **构建时**：8GB 内存，2-4 核 CPU，20-30GB 磁盘空间
- **运行时**：1-2GB 内存，1 核 CPU，5-10GB 磁盘空间（取决于链数据）

### 优化建议

1. **使用多阶段构建**：已实现，最终镜像只包含二进制文件
2. **利用 Docker 缓存**：修改代码时，只有变更的层会重新构建
3. **清理未使用的镜像**：
   ```bash
   docker system prune -a
   ```

## 🔐 安全建议

1. **使用非 root 用户**：镜像已配置，运行在 `substrate` 用户下
2. **最小化镜像**：使用 `parity/base-bin` 作为基础镜像，只包含必要文件
3. **限制资源**：使用 `docker run --memory=2g --cpus=1` 限制资源使用

## 📚 更多资源

- [Docker 官方文档](https://docs.docker.com/)
- [Substrate 官方文档](https://docs.substrate.io/)
- [Polkadot-JS Apps](https://polkadot.js.org/apps/)

## 💡 提示

- **首次构建**：需要下载基础镜像和编译所有依赖，可能需要 30 分钟到 1 小时
- **后续构建**：利用 Docker 缓存，通常只需要几分钟
- **开发建议**：使用 `docker-compose` 管理多个服务（如节点 + 数据库）
- **生产部署**：考虑使用 Kubernetes 或 Docker Swarm 进行容器编排


