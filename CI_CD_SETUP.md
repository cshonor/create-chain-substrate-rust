# CI/CD 配置指南

## 概述

本项目配置了 GitHub Actions 自动化 CI/CD 流程：

- **CI (持续集成)**: 每次 push 代码时自动构建和测试
- **CD (持续部署)**: push 到主分支时自动部署到服务器

## 工作流说明

### 1. CI - Build and Test (`ci.yml`)

**触发条件**:
- Push 到 `main`、`master` 或 `rebuld` 分支
- 创建 Pull Request

**执行内容**:
- ✅ 代码格式检查 (`cargo fmt`)
- ✅ 代码质量检查 (`cargo clippy`)
- ✅ 构建发布版本
- ✅ 上传构建产物

### 2. CD - Deploy to Server (`deploy.yml`)

**触发条件**:
- Push 到 `main`、`master` 或 `rebuld` 分支
- 创建版本标签 (`v*`)
- 手动触发 (workflow_dispatch)

**执行内容**:
- 🔨 构建发布版本
- 📤 上传二进制文件到服务器
- 🚀 自动部署并重启服务

### 3. Build Only (`build-only.yml`)

**触发条件**:
- Push 到 `develop` 或 `feature/*` 分支
- 手动触发

**执行内容**:
- 🔨 仅构建，不部署

## 配置步骤

### 步骤1：配置 GitHub Secrets

在 GitHub 仓库设置中添加以下 Secrets：

1. 进入仓库 → **Settings** → **Secrets and variables** → **Actions**
2. 点击 **New repository secret**，添加以下密钥：

| Secret 名称 | 说明 | 示例值 |
|------------|------|--------|
| `SERVER_HOST` | 服务器公网IP | `8.148.205.248` |
| `SERVER_USER` | SSH 用户名 | `root` 或 `dev` |
| `SERVER_PORT` | SSH 端口（可选） | `22` |
| `SSH_PRIVATE_KEY` | SSH 私钥 | 见下方说明 |

### 步骤2：生成 SSH 密钥对

**在本地执行**：

```bash
# 1. 生成 SSH 密钥对（如果还没有）
ssh-keygen -t rsa -b 4096 -C "github-actions" -f ~/.ssh/github_actions_deploy

# 2. 查看公钥内容
cat ~/.ssh/github_actions_deploy.pub

# 3. 将公钥添加到服务器
ssh-copy-id -i ~/.ssh/github_actions_deploy.pub root@8.148.205.248

# 或手动添加：
# ssh root@8.148.205.248
# echo "你的公钥内容" >> ~/.ssh/authorized_keys
```

**在 GitHub 中添加私钥**：

```bash
# 查看私钥内容
cat ~/.ssh/github_actions_deploy

# 复制私钥内容，添加到 GitHub Secrets 的 SSH_PRIVATE_KEY
```

### 步骤3：在服务器上准备部署目录

```bash
# SSH 连接到服务器
ssh root@8.148.205.248

# 创建部署目录
mkdir -p ~/deployments/substrate
mkdir -p ~/deployments/backups

# 上传部署脚本
# 在本地执行：
scp scripts/deploy.sh root@8.148.205.248:~/deployments/substrate/
```

### 步骤4：配置 systemd 服务（可选）

创建服务文件 `/etc/systemd/system/substrate-node.service`：

```bash
sudo nano /etc/systemd/system/substrate-node.service
```

内容：

```ini
[Unit]
Description=Substrate Node
After=network.target

[Service]
Type=simple
User=root
WorkingDirectory=/root
ExecStart=/usr/local/bin/minimal-template-node \
  --chain dev \
  --base-path /root/substrate-data \
  --name MySubstrateNode \
  --rpc-external \
  --ws-external \
  --rpc-cors all \
  --port 30333 \
  --ws-port 9944
Restart=always
RestartSec=10
StandardOutput=journal
StandardError=journal

[Install]
WantedBy=multi-user.target
```

启用服务：

```bash
sudo systemctl daemon-reload
sudo systemctl enable substrate-node
# 首次启动
sudo systemctl start substrate-node
```

## 使用方法

### 自动部署

1. **Push 代码到主分支**：
   ```bash
   git add .
   git commit -m "Update code"
   git push origin main
   ```

2. **查看部署状态**：
   - 进入 GitHub 仓库 → **Actions** 标签页
   - 查看最新的 workflow 运行状态

3. **验证部署**：
   ```bash
   ssh root@8.148.205.248
   systemctl status substrate-node
   journalctl -u substrate-node -f
   ```

### 手动触发部署

1. 进入 GitHub 仓库 → **Actions** → **CD - Deploy to Server**
2. 点击 **Run workflow**
3. 选择分支和环境，点击 **Run workflow**

### 创建版本标签触发部署

```bash
# 创建版本标签
git tag -a v1.0.0 -m "Release version 1.0.0"
git push origin v1.0.0
```

## 工作流文件说明

### `.github/workflows/ci.yml`
- 每次 push 和 PR 时运行
- 执行代码检查和构建
- 不上传产物到服务器

### `.github/workflows/deploy.yml`
- 仅在主分支 push 时运行
- 构建并部署到服务器
- 需要配置 SSH 密钥

### `.github/workflows/build-only.yml`
- 开发分支构建
- 仅构建，不部署

## 安全建议

1. **SSH 密钥安全**：
   - 使用专用的部署密钥，不要使用个人 SSH 密钥
   - 定期轮换密钥
   - 限制密钥权限（仅允许部署相关操作）

2. **服务器安全**：
   - 使用非 root 用户部署（创建专用部署用户）
   - 配置防火墙规则
   - 定期更新系统

3. **GitHub Secrets**：
   - 不要将 Secrets 提交到代码仓库
   - 定期检查 Secrets 的使用情况

## 故障排查

### 问题1：SSH 连接失败

```bash
# 检查 SSH 密钥是否正确
ssh -i ~/.ssh/github_actions_deploy root@8.148.205.248

# 检查服务器 SSH 配置
ssh root@8.148.205.248 "cat ~/.ssh/authorized_keys"
```

### 问题2：部署失败

```bash
# 查看 GitHub Actions 日志
# 在 Actions 页面点击失败的 workflow，查看详细日志

# 手动测试部署脚本
ssh root@8.148.205.248
cd ~/deployments/substrate
./deploy.sh
```

### 问题3：服务无法启动

```bash
# 检查服务状态
sudo systemctl status substrate-node

# 查看服务日志
sudo journalctl -u substrate-node -n 100

# 检查二进制文件权限
ls -lh /usr/local/bin/minimal-template-node
```

## 优化建议

1. **构建缓存**：已配置 Cargo 缓存，加速后续构建
2. **并行构建**：GitHub Actions 使用 4 个并行任务
3. **构建矩阵**：可以添加多平台构建（Linux、macOS、Windows）
4. **通知**：可以添加 Slack/Discord 通知，部署成功/失败时提醒

## 下一步

- ✅ 配置 GitHub Secrets
- ✅ 生成并配置 SSH 密钥
- ✅ 测试 CI/CD 流程
- 🔄 优化构建速度
- 🔄 添加更多测试
- 🔄 配置监控和告警

