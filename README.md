<div align="center">

# Polkadot SDK 最小模板

<img height="70px" alt="Polkadot SDK Logo" src="https://github.com/paritytech/polkadot-sdk/raw/master/docs/images/Polkadot_Logo_Horizontal_Pink_White.png#gh-dark-mode-only"/>
<img height="70px" alt="Polkadot SDK Logo" src="https://github.com/paritytech/polkadot-sdk/raw/master/docs/images/Polkadot_Logo_Horizontal_Pink_Black.png#gh-light-mode-only"/>

> 这是一个基于 Polkadot SDK 创建区块链的最小模板。
>
> 此模板会在主 [Polkadot SDK 单体仓库](https://github.com/paritytech/polkadot-sdk) 发布后自动更新。

</div>

## 目录

- [简介](#简介)

- [模板结构](#模板结构)

- [快速开始](#快速开始)

- [启动最小模板链](#启动最小模板链)

  - [最小模板节点](#最小模板节点)
  - [使用最小模板节点的 Zombienet](#使用最小模板节点的-zombienet)
  - [连接 Polkadot-JS Apps 前端](#连接-polkadot-js-apps-前端)
  - [要点](#要点)

- [贡献](#贡献)

- [获取帮助](#获取帮助)

## 简介

- 🤏 这个模板是一个最小化（在复杂性和组件数量方面）的区块链节点构建模板。

- 🔧 其运行时配置了一个自定义 pallet 作为起点，以及一些现成的 pallet，例如 [Balances pallet](https://paritytech.github.io/polkadot-sdk/master/pallet_balances/index.html)。

- 👤 该模板未配置共识机制 - 最适合用于单节点网络的实验。

## 模板结构

基于 Polkadot SDK 的项目（如本项目）包含：

- 🧮 [运行时](./runtime/README.md) - 区块链的核心逻辑。
- 🎨 [Pallets](./pallets/README.md) - 用于构建运行时的组件。
- 💿 [节点](./node/README.md) - 二进制应用程序（不在 cargo default-members 列表中，除非构建整个工作区，否则不会编译）。

## 快速开始

- 🦀 该模板使用 Rust 语言。

- 👉 请查看适用于您系统的 [Rust 安装说明](https://www.rust-lang.org/tools/install)。

- 🛠️ 根据您的操作系统和 Rust 版本，可能需要额外的包来编译此模板 - 请注意 Rust 编译器的输出。

获取最小模板代码。

```sh
git clone https://github.com/paritytech/polkadot-sdk-minimal-template.git minimal-template

cd minimal-template
```

## 启动最小模板链

### 最小模板节点

#### 构建节点和运行时

```sh
cargo build --workspace --release
```

🐳 或者，构建 Docker 镜像，该镜像会构建所有工作区成员，并以节点二进制文件作为入口点：

```sh
docker build . -t polkadot-sdk-minimal-template
```

#### 启动 `minimal-template-node`

`minimal-template-node` 依赖于 `minimal-template-runtime`。它将使用 `minimal_template_runtime::WASM_BINARY` 常量（该常量将 WASM 二进制文件保存为字节数组）来构建链规范，同时启动。

```sh
<target/release/path/to/minimal-template-node> --tmp --consensus manual-seal-3000
# 或通过 docker
docker run --rm polkadot-sdk-minimal-template
```

#### 使用 `minimal-template-node` 的 Zombienet

对于这个，我们只需要安装 `zombienet` 并运行：

```sh
zombienet --provider native spawn zombienet-multi-node.toml
```

### 连接 Polkadot-JS Apps 前端

- 🌐 您可以使用 [Polkadot/Substrate Portal](https://polkadot.js.org/apps/#/explorer?rpc=ws://localhost:9944) 的托管版本与本地节点交互。

- 🪐 在 [IPFS](https://dotapps.io/) 上也提供了托管版本。

- 🧑‍🔧 您还可以在 [`polkadot-js/apps`](https://github.com/polkadot-js/apps) 仓库中找到源代码和托管自己实例的说明。

### 要点

之前最小模板的开发链：

- ❌ 在多节点设置中启动会产生分叉，因为最小模板缺少共识机制。
- 🧹 不会持久化状态。
- 💰 预配置了包含多个预充值开发账户的创世状态。
- 🧑‍⚖️ 一个开发账户（`ALICE`）用作 `sudo` 账户。

## 贡献

- 🔄 此模板会在主 [Polkadot SDK 单体仓库](https://github.com/paritytech/polkadot-sdk) 发布后自动更新。

- ➡️ 任何拉取请求都应指向此[源](https://github.com/paritytech/polkadot-sdk/tree/master/templates/minimal)。

- 😇 请参考单体仓库的[贡献指南](https://github.com/paritytech/polkadot-sdk/blob/master/docs/contributor/CONTRIBUTING.md)和[行为准则](https://github.com/paritytech/polkadot-sdk/blob/master/docs/contributor/CODE_OF_CONDUCT.md)。

## 获取帮助

- 🧑‍🏫 要了解 Polkadot 的一般信息，[docs.Polkadot.com](https://docs.polkadot.com/) 网站是一个很好的起点。

- 🧑‍🔧 对于技术介绍，[这里](https://github.com/paritytech/polkadot-sdk#-documentation)是 Polkadot SDK 文档资源。

- 👥 此外，还有 [GitHub issues](https://github.com/paritytech/polkadot-sdk/issues) 和 [Substrate StackExchange](https://substrate.stackexchange.com/)。
- 👥 您也可以在 [官方 Polkadot Discord 服务器](https://polkadot-discord.w3f.tools/) 上联系
- 🧑 在 [Telegram](https://t.me/substratedevs) 上联系，获取更多问题和讨论
