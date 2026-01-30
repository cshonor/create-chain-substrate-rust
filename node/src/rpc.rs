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

//! 节点特定的 RPC 方法集合
//! Substrate 提供了 `sc-rpc` crate，定义了 Substrate 节点使用的核心 RPC 层
//! 此文件扩展了那些 RPC 定义，添加了特定于此项目运行时配置的功能

#![warn(missing_docs)]

use jsonrpsee::RpcModule;
use minimal_template_runtime::interface::{AccountId, Nonce, OpaqueBlock};
use polkadot_sdk::{
	sc_transaction_pool_api::TransactionPool,
	sp_blockchain::{Error as BlockChainError, HeaderBackend, HeaderMetadata},
	*,
};
use std::sync::Arc;

/// 完整客户端依赖项
/// 包含 RPC 服务所需的所有依赖
pub struct FullDeps<C, P> {
	/// 要使用的客户端实例
	pub client: Arc<C>,
	/// 交易池实例
	pub pool: Arc<P>,
}

#[docify::export]
/// 实例化所有完整的 RPC 扩展
/// 创建并配置所有 RPC 方法模块
pub fn create_full<C, P>(
	deps: FullDeps<C, P>,
) -> Result<RpcModule<()>, Box<dyn std::error::Error + Send + Sync>>
where
	C: Send
		+ Sync
		+ 'static
		+ sp_api::ProvideRuntimeApi<OpaqueBlock>
		+ HeaderBackend<OpaqueBlock>
		+ HeaderMetadata<OpaqueBlock, Error = BlockChainError>
		+ 'static,
	C::Api: sp_block_builder::BlockBuilder<OpaqueBlock>,
	C::Api: substrate_frame_rpc_system::AccountNonceApi<OpaqueBlock, AccountId, Nonce>,
	P: TransactionPool + 'static,
{
	use polkadot_sdk::substrate_frame_rpc_system::{System, SystemApiServer};
	let mut module = RpcModule::new(());
	let FullDeps { client, pool } = deps;

	// 添加系统 RPC 方法（账户 nonce 等）
	module.merge(System::new(client.clone(), pool.clone()).into_rpc())?;

	Ok(module)
}
