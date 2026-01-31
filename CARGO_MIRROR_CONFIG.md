# Cargo 镜像源配置指南

## 配置位置

- **项目级配置**：`.cargo/config.toml`（仅影响当前项目）
- **全局配置**：`~/.cargo/config.toml`（影响所有项目）

## 推荐配置（中科大镜像）

### 在服务器上配置（WSL 或 Linux）

```bash
# 创建配置目录
mkdir -p ~/.cargo

# 创建配置文件
cat > ~/.cargo/config.toml << 'EOF'
[source.crates-io]
replace-with = "ustc"

[source.ustc]
registry = "https://mirrors.ustc.edu.cn/crates.io-index"

[net]
git-fetch-with-cli = true
retry = 3
EOF
```

### 在 Windows 上配置

```powershell
# 创建配置目录
New-Item -ItemType Directory -Force -Path "$env:USERPROFILE\.cargo"

# 创建配置文件
@"
[source.crates-io]
replace-with = "ustc"

[source.ustc]
registry = "https://mirrors.ustc.edu.cn/crates.io-index"

[net]
git-fetch-with-cli = true
retry = 3
"@ | Out-File -FilePath "$env:USERPROFILE\.cargo\config.toml" -Encoding utf8
```

## 多个镜像源配置（备用方案）

如果需要配置多个镜像源作为备用：

```toml
[source.crates-io]
replace-with = "ustc"

# 中科大镜像（主用）
[source.ustc]
registry = "https://mirrors.ustc.edu.cn/crates.io-index"

# 阿里云镜像（备用）
[source.aliyun]
registry = "sparse+https://mirrors.aliyun.com/crates.io-index/"

# 清华大学镜像（备用）
[source.tuna]
registry = "https://mirrors.tuna.tsinghua.edu.cn/git/crates.io-index.git"

# 上海交大镜像（备用）
[source.sjtu]
registry = "https://mirrors.sjtug.sjtu.edu.cn/git/crates.io-index/"

[net]
git-fetch-with-cli = true
retry = 3
```

**注意**：Cargo 一次只能使用一个镜像源。如果需要切换，修改 `replace-with` 的值即可。

## 切换镜像源

```bash
# 切换到中科大镜像
sed -i 's/replace-with = ".*"/replace-with = "ustc"/' ~/.cargo/config.toml

# 切换到阿里云镜像
sed -i 's/replace-with = ".*"/replace-with = "aliyun"/' ~/.cargo/config.toml

# 切换到清华大学镜像
sed -i 's/replace-with = ".*"/replace-with = "tuna"/' ~/.cargo/config.toml

# 切换回官方源（删除 replace-with 或注释掉）
sed -i 's/^\[source.crates-io\]/# [source.crates-io]/' ~/.cargo/config.toml
sed -i 's/^replace-with/# replace-with/' ~/.cargo/config.toml
```

## 验证配置

```bash
# 检查配置
cat ~/.cargo/config.toml

# 测试下载（会显示使用的镜像源）
cargo search substrate --limit 1
```

## 常见问题

### Q: 镜像源配置后仍然下载失败？

1. **检查网络连接**：
   ```bash
   curl -I https://mirrors.ustc.edu.cn/crates.io-index
   ```

2. **清理缓存后重试**：
   ```bash
   rm -rf ~/.cargo/registry/cache
   cargo build
   ```

3. **切换到其他镜像源**：
   ```bash
   # 尝试阿里云
   sed -i 's/replace-with = ".*"/replace-with = "aliyun"/' ~/.cargo/config.toml
   ```

### Q: 如何临时使用官方源？

```bash
# 备份当前配置
cp ~/.cargo/config.toml ~/.cargo/config.toml.bak

# 临时禁用镜像
mv ~/.cargo/config.toml ~/.cargo/config.toml.disabled

# 使用官方源构建
cargo build

# 恢复镜像配置
mv ~/.cargo/config.toml.disabled ~/.cargo/config.toml
```

### Q: 项目级配置和全局配置哪个优先？

- **项目级配置**（`.cargo/config.toml`）优先于全局配置
- 如果项目级配置存在，会覆盖全局配置

## 推荐做法

1. **开发环境**：使用全局配置（`~/.cargo/config.toml`），所有项目共享
2. **CI/CD**：项目级配置（`.cargo/config.toml`），确保构建环境一致
3. **生产环境**：根据网络情况选择，国内服务器推荐使用镜像源

