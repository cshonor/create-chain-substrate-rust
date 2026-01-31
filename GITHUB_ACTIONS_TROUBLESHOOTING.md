# GitHub Actions 部署故障排查指南

## ⚠️ 核心要点

本指南基于 Substrate + GitHub Actions 的实际部署经验，覆盖了 90% 以上的常见失败场景。

## 常见问题

### 问题 1：WASM 编译失败 - rust-src 组件不匹配

**错误信息**：
```
error: failed to run custom build command for `substrate-wasm-builder`
Cannot compile the WASM runtime: no standard library sources found
```

**根本原因**：
- Substrate 的 WASM 编译要求 `rust-src` 组件必须和 `WASM_BUILD_TOOLCHAIN` 指定的 nightly 版本匹配
- **如果只给 stable 装 rust-src，nightly 没有的话，依然会报错**

**正确解决方案**：
```yaml
- name: Install Rust & Nightly (带 rust-src)
  uses: dtolnay/rust-toolchain@stable
  with:
    toolchain: stable
    components: rustfmt, clippy, rust-src
    targets: wasm32-unknown-unknown

- name: 给 Nightly 也装 rust-src 和 WASM 目标
  run: |
    rustup toolchain install nightly-2024-07-01 --component rust-src
    rustup target add wasm32-unknown-unknown --toolchain nightly-2024-07-01
```

**关键点**：`--component rust-src` 必须同时给 nightly 安装！

### 问题 2：SSH 连接失败 - 私钥权限和指纹问题

**错误信息**：
```
Error: Permission denied (publickey)
Host key verification failed
```

**根本原因**：
1. **私钥权限过宽**：GitHub Actions runner 里私钥默认权限可能是 644，SSH 要求必须 600
2. **服务器首次连接需要确认指纹**：GitHub Actions 会卡住等待确认

**正确解决方案**：
```yaml
- name: 配置 SSH（解决指纹/权限问题）
  run: |
    mkdir -p ~/.ssh
    echo "${{ secrets.SSH_PRIVATE_KEY }}" > ~/.ssh/deploy_key
    chmod 600 ~/.ssh/deploy_key  # 关键：SSH 要求私钥权限必须是 600
    ssh-keyscan -p ${{ env.SERVER_PORT }} ${{ env.SERVER_HOST }} >> ~/.ssh/known_hosts
    chmod 644 ~/.ssh/known_hosts

# 后续所有 SSH/scp 命令都要指定私钥路径
- name: 上传文件
  run: |
    scp -i ~/.ssh/deploy_key -P ${{ env.SERVER_PORT }} \
      target/release/minimal-template-node \
      ${{ env.SERVER_USER }}@${{ env.SERVER_HOST }}:~/deployments/substrate/
```

**关键点**：
- 私钥权限必须是 `600`，不能是 `644` 或 `755`
- 使用 `-i ~/.ssh/deploy_key` 明确指定私钥路径
- 使用 `ssh-keyscan` 预先添加服务器指纹到 `known_hosts`

### 问题 3：构建超时 - CPU 核心数不匹配

**错误信息**：
```
Error: The operation was canceled.
```

**根本原因**：
- GitHub Actions 免费版的 runner 是 **2 核 CPU**
- 设置 `CARGO_BUILD_JOBS=4` 会导致 CPU 上下文切换反而变慢
- **建议 JOBS 数等于 CPU 核心数（2）**

**正确解决方案**：
```yaml
- name: Build release
  run: |
    export CARGO_BUILD_JOBS=2  # 修正为 2，匹配 runner 核心数
    export RUSTFLAGS='-A useless_deprecated'
    export WASM_BUILD_TOOLCHAIN=nightly-2024-07-01
    cargo build --release --locked
  timeout-minutes: 60
```

**性能对比**：
- `CARGO_BUILD_JOBS=4`（2 核）：上下文切换开销大，实际更慢
- `CARGO_BUILD_JOBS=2`（2 核）：充分利用 CPU，编译更快

### 问题 4：Cargo 缓存无效 - target 目录缓存问题

**错误信息**：
```
缓存命中但构建仍然很慢
```

**根本原因**：
- Substrate 编译的 `target` 目录会因 Rust 版本/环境变量变化失效
- GitHub Actions 缓存有 10GB 限制，缓存整个 `target` 容易超限
- **建议拆分缓存：只缓存依赖，不缓存 target**

**正确解决方案**：
```yaml
# 只缓存依赖（高效，不会因环境变化失效）
- name: Cache Cargo 依赖（更高效）
  uses: actions/cache@v4
  with:
    path: |
      ~/.cargo/registry/index
      ~/.cargo/registry/cache
      ~/.cargo/git/db
    key: ${{ runner.os }}-cargo-deps-${{ hashFiles('**/Cargo.lock') }}
    restore-keys: |
      ${{ runner.os }}-cargo-deps-

# target 单独缓存（可选，注意：环境变化时要失效）
- name: Cache Target
  uses: actions/cache@v4
  with:
    path: target
    key: ${{ runner.os }}-target-${{ hashFiles('**/Cargo.lock') }}-${{ env.WASM_BUILD_TOOLCHAIN }}
    restore-keys: |
      ${{ runner.os }}-target-
```

**关键点**：
- 依赖缓存：`~/.cargo/registry` 和 `~/.cargo/git`，稳定且高效
- target 缓存：单独缓存，key 包含环境变量，环境变化时自动失效

### 问题 5：服务器端文件权限/SELinux 问题

**错误信息**：
```
Permission denied
./deploy.sh: cannot execute binary file
```

**根本原因**：
- 服务器开启了 SELinux，阻止二进制文件执行
- 文件权限未正确设置

**正确解决方案**：
```yaml
- name: 部署（处理 SELinux + 赋权）
  run: |
    ssh -v -i ~/.ssh/deploy_key -p ${{ env.SERVER_PORT }} ${{ env.SERVER_USER }}@${{ env.SERVER_HOST }} << 'EOF'
      set -x  # 显示执行的每一条命令
      # 临时关闭 SELinux（如果启用）
      if command -v setenforce &> /dev/null; then
        setenforce 0 || true
      fi
      cd ~/deployments/substrate
      chmod +x minimal-template-node  # 关键：给二进制文件执行权限
      chmod +x deploy.sh
      ./deploy.sh
    EOF
```

**关键点**：
- 使用 `setenforce 0` 临时关闭 SELinux（生产环境建议永久配置）
- 确保二进制文件和脚本都有执行权限
- 使用 `set -x` 显示详细执行日志

### 问题 6：GitHub Actions Runner 磁盘空间不足

**错误信息**：
```
No space left on device
```

**根本原因**：
- Substrate 编译需要约 10GB 磁盘空间
- GitHub Actions 免费 runner 默认磁盘可能不够

**正确解决方案**：
```yaml
- name: 清理 Runner 磁盘空间（前置）
  run: |
    sudo rm -rf /usr/share/dotnet /usr/local/lib/android /opt/ghc
    df -h  # 查看剩余空间
```

**清理内容**：
- `/usr/share/dotnet`：.NET SDK（通常不需要）
- `/usr/local/lib/android`：Android SDK（通常不需要）
- `/opt/ghc`：Haskell 编译器（通常不需要）

### 问题 7：Cargo.lock 版本不匹配

**错误信息**：
```
error: the lock file needs to be updated
```

**根本原因**：
- 本地 `Cargo.lock` 是 Windows/WSL 生成的
- 和 Linux runner 的 `Cargo.lock` 可能有差异

**正确解决方案**：
```yaml
- name: 重新生成 Cargo.lock（确保兼容 Linux）
  run: |
    rm -f Cargo.lock
    cargo generate-lockfile
```

**关键点**：
- 在 Linux runner 中重新生成 `Cargo.lock`，确保平台兼容性
- 如果项目需要跨平台，考虑在 CI 中统一生成

### 问题 8：二进制文件不存在

**错误信息**：
```
Error: 二进制文件不存在
scp: target/release/minimal-template-node: No such file or directory
```

**原因**：
- 构建失败但没有报错
- 二进制文件路径不正确
- 工作区配置问题

**解决方案**：
1. **添加构建验证步骤**：
```yaml
- name: Check binary exists
  run: |
    if [ ! -f target/release/minimal-template-node ]; then
      echo "错误: 二进制文件不存在"
      ls -la target/release/ || echo "target/release 目录不存在"
      exit 1
    fi
    ls -lh target/release/minimal-template-node
```

2. **检查构建命令**：
```yaml
# 如果使用工作区，确保构建所有成员
cargo build --workspace --release --locked
```

## 调试技巧

### 1. 实时输出编译日志

Substrate 编译日志很长，GitHub Actions 默认会截断，添加实时输出：

```yaml
- name: Build release（实时输出日志）
  run: |
    export CARGO_TERM_VERBOSE=true  # 显示详细日志
    cargo build --release --locked 2>&1 | tee build.log
  timeout-minutes: 60

- name: 上传编译日志（失败时）
  uses: actions/upload-artifact@v4
  if: failure()
  with:
    name: build-log
    path: build.log
```

### 2. 远程执行命令时输出详细日志

```yaml
- name: 部署（输出详细日志）
  run: |
    ssh -v -i ~/.ssh/deploy_key -p ${{ env.SERVER_PORT }} ${{ env.SERVER_USER }}@${{ env.SERVER_HOST }} << 'EOF'
      set -x  # 显示执行的每一条命令
      cd ~/deployments/substrate
      ./deploy.sh
    EOF
```

**关键参数**：
- `-v`：SSH 详细模式，显示连接过程
- `set -x`：bash 详细模式，显示每条命令

### 3. 添加调试输出

在关键步骤添加调试信息：

```yaml
- name: Debug info
  run: |
    echo "Rust version:"
    rustc --version
    echo "Cargo version:"
    cargo --version
    echo "Available toolchains:"
    rustup toolchain list
    echo "Environment:"
    env | grep -E "(RUST|CARGO|WASM)"
    echo "Disk space:"
    df -h
    echo "Memory:"
    free -h
```

### 4. 本地复现问题

在本地 WSL 或 Linux 环境中复现：

```bash
# 使用相同的环境变量
export CARGO_BUILD_JOBS=2  # 注意：匹配 CPU 核心数
export RUSTFLAGS='-A useless_deprecated'
export WASM_BUILD_TOOLCHAIN=nightly-2024-07-01

# 执行相同的构建命令
cargo build --release --locked
```

### 5. 检查构建产物

```yaml
- name: List build artifacts
  run: |
    find target/release -type f -executable -ls
    file target/release/minimal-template-node
    ls -lh target/release/minimal-template-node
```

## 最佳实践总结

### 1. Rust 工具链配置
- ✅ **同时给 stable 和 nightly 安装 rust-src**
- ✅ 使用 `--component rust-src` 明确指定
- ✅ 验证工具链安装：`rustup toolchain list`

### 2. SSH 配置
- ✅ **私钥权限必须是 600**：`chmod 600 ~/.ssh/deploy_key`
- ✅ 使用 `ssh-keyscan` 预先添加服务器指纹
- ✅ 所有 SSH/scp 命令明确指定私钥路径：`-i ~/.ssh/deploy_key`

### 3. 构建优化
- ✅ **CARGO_BUILD_JOBS 等于 CPU 核心数**（GitHub Actions 免费版是 2）
- ✅ 拆分缓存：依赖缓存 + target 缓存（可选）
- ✅ 清理 runner 磁盘空间（删除不需要的 SDK）

### 4. 错误处理
- ✅ 添加超时设置：`timeout-minutes: 60`
- ✅ 验证步骤：检查二进制文件存在
- ✅ 失败时上传日志：`if: failure()`

### 5. 服务器部署
- ✅ 处理 SELinux：`setenforce 0`（临时）或永久配置
- ✅ 确保文件权限：`chmod +x`
- ✅ 使用详细日志：`set -x` 和 `ssh -v`

## 完整工作流示例

参考 `.github/workflows/deploy.yml`，已应用所有最佳实践。

## 相关资源

- [GitHub Actions 文档](https://docs.github.com/en/actions)
- [Substrate 构建文档](https://docs.substrate.io/main-docs/install/)
- [Rust 工具链管理](https://rust-lang.github.io/rustup/)
- [SSH 密钥管理最佳实践](https://docs.github.com/en/authentication/connecting-to-github-with-ssh)

## 问题反馈

如果遇到本指南未覆盖的问题，请提供：
1. 完整的错误日志
2. GitHub Actions workflow 文件
3. 失败的具体步骤

这样可以定制化修正步骤。
