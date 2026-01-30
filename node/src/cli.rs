// This file is part of Substrate.

// Copyright (C) Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use polkadot_sdk::*;

/// 共识机制类型
/// 定义节点支持的共识算法类型
#[derive(Debug, Clone)]
pub enum Consensus {
	/// 手动出块，参数为出块间隔（毫秒）
	ManualSeal(u64),
	/// 即时出块（收到交易立即出块）
	InstantSeal,
	/// 无共识（仅用于测试）
	None,
}

/// 从字符串解析共识类型
/// 支持的格式：
/// - "instant-seal" -> InstantSeal
/// - "manual-seal-3000" -> ManualSeal(3000)
/// - "none" -> None
impl std::str::FromStr for Consensus {
	type Err = String;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Ok(if s == "instant-seal" {
			Consensus::InstantSeal
		} else if let Some(block_time) = s.strip_prefix("manual-seal-") {
			Consensus::ManualSeal(block_time.parse().map_err(|_| "invalid block time")?)
		} else if s.to_lowercase() == "none" {
			Consensus::None
		} else {
			return Err("incorrect consensus identifier".into());
		})
	}
}

/// 命令行参数结构体
/// 定义节点支持的所有命令行选项
#[derive(Debug, clap::Parser)]
pub struct Cli {
	/// 子命令（可选）
	#[command(subcommand)]
	pub subcommand: Option<Subcommand>,

	/// 共识机制类型，默认为手动出块，间隔 3000 毫秒
	#[clap(long, default_value = "manual-seal-3000")]
	pub consensus: Consensus,

	/// 运行节点的通用参数（如 --dev, --tmp 等）
	#[clap(flatten)]
	pub run: sc_cli::RunCmd,
}

/// 子命令枚举
/// 定义节点支持的所有子命令
#[derive(Debug, clap::Subcommand)]
pub enum Subcommand {
	/// 密钥管理 CLI 工具
	#[command(subcommand)]
	Key(sc_cli::KeySubcommand),

	/// 构建链规范
	/// 已弃用：`build-spec` 命令将在 2026/04/01 后移除，请使用 `export-chain-spec` 命令
	#[deprecated(
		note = "build-spec command will be removed after 1/04/2026. Use export-chain-spec command instead"
	)]
	BuildSpec(sc_cli::BuildSpecCmd),

	/// 导出链规范
	ExportChainSpec(sc_cli::ExportChainSpecCmd),

	/// 验证区块
	CheckBlock(sc_cli::CheckBlockCmd),

	/// 导出区块
	ExportBlocks(sc_cli::ExportBlocksCmd),

	/// 将指定区块的状态导出为链规范
	ExportState(sc_cli::ExportStateCmd),

	/// 导入区块
	ImportBlocks(sc_cli::ImportBlocksCmd),

	/// 删除整个链数据
	PurgeChain(sc_cli::PurgeChainCmd),

	/// 将链回退到之前的状态
	Revert(sc_cli::RevertCmd),

	/// 数据库元列信息
	ChainInfo(sc_cli::ChainInfoCmd),
}
