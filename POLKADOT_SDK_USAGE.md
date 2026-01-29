# Polkadot SDK 使用说明

## 当前配置

你的项目当前使用的是 **crates.io 版本**的 polkadot-sdk：

```toml
[workspace.dependencies]
polkadot-sdk = { version = "1.0", default-features = false }
```

这意味着：
- ✅ **不需要本地 polkadot-sdk 代码**
- ✅ Cargo 会自动从 crates.io 下载依赖
- ✅ 更简单，无需管理本地代码
- ✅ 适合大多数开发场景

## 是否需要本地 polkadot-sdk 代码？

### 不需要本地代码的情况（推荐，当前配置）

**适用场景：**
- 标准开发，使用稳定版本
- 不需要修改 polkadot-sdk 源码
- 希望项目结构简单

**操作：**
- 可以删除 `substrate/polkadot-sdk/` 目录
- 保持当前 `Cargo.toml` 配置不变

### 需要本地代码的情况

**适用场景：**
- 需要修改 polkadot-sdk 源码进行调试
- 需要使用最新开发版本（未发布到 crates.io）
- 需要特定分支或提交

**配置方式：**

1. **使用本地路径**（推荐放在项目外部）：
   ```toml
   [workspace.dependencies]
   polkadot-sdk = { path = "../polkadot-sdk/umbrella", default-features = false }
   ```

2. **使用 Git 仓库**：
   ```toml
   [workspace.dependencies]
   polkadot-sdk = { git = "https://github.com/paritytech/polkadot-sdk.git", branch = "master", default-features = false }
   ```

## 推荐的项目结构

### 方案 1：使用 crates.io（推荐）

```
rs/
├── substrate/          # 你的项目
│   ├── Cargo.toml      # 使用 version = "1.0"
│   ├── node/
│   ├── runtime/
│   └── pallets/
└── polkadot-sdk/       # 可以删除，或保留用于参考
```

### 方案 2：使用本地代码

```
rs/
├── substrate/          # 你的项目
│   ├── Cargo.toml      # 使用 path = "../polkadot-sdk/umbrella"
│   ├── node/
│   ├── runtime/
│   └── pallets/
└── polkadot-sdk/       # 完整的 SDK 代码（必需）
    └── umbrella/       # 指向这个目录
```

## 建议

对于大多数开发场景，**推荐使用 crates.io 版本**（当前配置）：
- 更简单，无需管理本地代码
- 自动处理版本更新
- 减少项目体积

如果需要本地代码，建议：
- 将 `polkadot-sdk` 放在项目外部（如 `/mnt/c/Users/12392/Desktop/rs/polkadot-sdk`）
- 在 `Cargo.toml` 中使用相对路径引用
- 不要将 `polkadot-sdk` 提交到项目仓库

