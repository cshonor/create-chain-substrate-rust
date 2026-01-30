# SSH 配置指南

## SSH Config 文件位置

Windows: `C:\Users\你的用户名\.ssh\config`  
Mac/Linux: `~/.ssh/config`

## 使用方法

配置完成后，可以使用简短的别名连接服务器：

```bash
# 之前需要输入完整命令
ssh root@8.148.205.248

# 现在只需要
ssh aliyun-ecs
```

## 配置说明

### 1. 基础服务器连接

```bash
ssh aliyun-ecs
```

### 2. GitHub Actions 部署连接

```bash
ssh github-actions-deploy
```

### 3. 开发用户连接

```bash
ssh aliyun-dev
```

## 生成 SSH 密钥对（用于 CI/CD）

### 在 Windows PowerShell 中执行：

```powershell
# 1. 生成专用部署密钥
ssh-keygen -t rsa -b 4096 -C "github-actions-deploy" -f $env:USERPROFILE\.ssh\github_actions_deploy

# 2. 查看公钥（需要添加到服务器）
cat $env:USERPROFILE\.ssh\github_actions_deploy.pub

# 3. 查看私钥（需要添加到 GitHub Secrets）
cat $env:USERPROFILE\.ssh\github_actions_deploy
```

### 将公钥添加到服务器：

```powershell
# 方法1：使用 ssh-copy-id（如果已安装 Git Bash）
ssh-copy-id -i $env:USERPROFILE\.ssh\github_actions_deploy.pub root@8.148.205.248

# 方法2：手动添加
# 1. 复制公钥内容
cat $env:USERPROFILE\.ssh\github_actions_deploy.pub

# 2. SSH 连接到服务器
ssh aliyun-ecs

# 3. 在服务器上执行
mkdir -p ~/.ssh
chmod 700 ~/.ssh
echo "你的公钥内容" >> ~/.ssh/authorized_keys
chmod 600 ~/.ssh/authorized_keys
```

### 测试连接：

```powershell
# 测试部署密钥连接
ssh -i $env:USERPROFILE\.ssh\github_actions_deploy github-actions-deploy

# 或使用配置的别名
ssh github-actions-deploy
```

## 配置 GitHub Secrets

1. 进入 GitHub 仓库 → **Settings** → **Secrets and variables** → **Actions**
2. 添加以下 Secrets：

| Secret 名称 | 值 |
|------------|-----|
| `SERVER_HOST` | `8.148.205.248` |
| `SERVER_USER` | `root` |
| `SERVER_PORT` | `22` |
| `SSH_PRIVATE_KEY` | 私钥内容（从 `github_actions_deploy` 文件复制） |

### 获取私钥内容：

```powershell
# 在 PowerShell 中执行
Get-Content $env:USERPROFILE\.ssh\github_actions_deploy
```

复制输出的全部内容（包括 `-----BEGIN OPENSSH PRIVATE KEY-----` 和 `-----END OPENSSH PRIVATE KEY-----`）

## 安全建议

1. **密钥权限**：
   - 确保私钥文件权限正确（Windows 通常不需要）
   - 不要将私钥提交到 Git 仓库

2. **服务器安全**：
   - 考虑创建专用部署用户（非 root）
   - 限制 SSH 密钥的权限

3. **定期轮换**：
   - 建议每 3-6 个月轮换一次密钥

## 常见问题

### Q: 连接被拒绝？
- 检查安全组是否开放 22 端口
- 检查服务器是否运行
- 检查用户名是否正确

### Q: 权限被拒绝？
- 确认公钥已正确添加到服务器的 `~/.ssh/authorized_keys`
- 检查服务器上的文件权限：
  ```bash
  chmod 700 ~/.ssh
  chmod 600 ~/.ssh/authorized_keys
  ```

### Q: 如何测试 GitHub Actions 的 SSH 连接？
```bash
# 使用部署密钥测试
ssh -i ~/.ssh/github_actions_deploy github-actions-deploy
```

## 下一步

1. ✅ 配置 SSH config 文件
2. ✅ 生成 SSH 密钥对
3. ✅ 将公钥添加到服务器
4. ✅ 配置 GitHub Secrets
5. 🔄 测试 CI/CD 流程

