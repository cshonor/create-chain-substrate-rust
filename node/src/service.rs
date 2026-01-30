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

use crate::cli::Consensus;
use futures::FutureExt;
use minimal_template_runtime::{interface::OpaqueBlock as Block, RuntimeApi};
use polkadot_sdk::{
	sc_client_api::backend::Backend,
	sc_executor::WasmExecutor,
	sc_service::{error::Error as ServiceError, Configuration, TaskManager},
	sc_telemetry::{Telemetry, TelemetryWorker},
	sc_transaction_pool_api::OffchainTransactionPoolFactory,
	sp_runtime::traits::Block as BlockT,
	*,
};
use std::sync::Arc;

/// Wasm 执行器的宿主函数类型
type HostFunctions = sp_io::SubstrateHostFunctions;

/// 完整客户端类型
/// 包含区块、运行时 API 和 Wasm 执行器
#[docify::export]
pub(crate) type FullClient =
	sc_service::TFullClient<Block, RuntimeApi, WasmExecutor<HostFunctions>>;

/// 完整后端类型
type FullBackend = sc_service::TFullBackend<Block>;
/// 最长链选择器类型
type FullSelectChain = sc_consensus::LongestChain<FullBackend, Block>;

/// 部分组件组装
/// 包含运行链操作子命令所需的足够组件（不需要完整节点）
pub type Service = sc_service::PartialComponents<
	FullClient,                                    // 完整客户端
	FullBackend,                                   // 后端存储
	FullSelectChain,                               // 链选择器
	sc_consensus::DefaultImportQueue<Block>,       // 导入队列
	sc_transaction_pool::TransactionPoolHandle<Block, FullClient>, // 交易池句柄
	Option<Telemetry>,                             // 遥测（可选）
>;

/// 创建部分服务组件
/// 用于运行链操作子命令（如导入区块、导出状态等）
pub fn new_partial(config: &Configuration) -> Result<Service, ServiceError> {
	// 初始化遥测（如果配置了遥测端点）
	let telemetry = config
		.telemetry_endpoints
		.clone()
		.filter(|x| !x.is_empty())
		.map(|endpoints| -> Result<_, sc_telemetry::Error> {
			let worker = TelemetryWorker::new(16)?;
			let telemetry = worker.handle().new_telemetry(endpoints);
			Ok((worker, telemetry))
		})
		.transpose()?;

	// 创建 Wasm 执行器
	let executor = sc_service::new_wasm_executor(&config.executor);

	// 创建客户端、后端、密钥库容器和任务管理器
	let (client, backend, keystore_container, task_manager) =
		sc_service::new_full_parts::<Block, RuntimeApi, _>(
			config,
			telemetry.as_ref().map(|(_, telemetry)| telemetry.handle()),
			executor,
		)?;
	let client = Arc::new(client);

	// 启动遥测工作线程
	let telemetry = telemetry.map(|(worker, telemetry)| {
		task_manager.spawn_handle().spawn("telemetry", None, worker.run());
		telemetry
	});

	// 创建最长链选择器
	let select_chain = sc_consensus::LongestChain::new(backend.clone());

	// 创建交易池
	let transaction_pool = Arc::from(
		sc_transaction_pool::Builder::new(
			task_manager.spawn_essential_handle(),
			client.clone(),
			config.role.is_authority().into(),
		)
		.with_options(config.transaction_pool.clone())
		.with_prometheus(config.prometheus_registry())
		.build(),
	);

	// 创建手动出块的导入队列
	let import_queue = sc_consensus_manual_seal::import_queue(
		Box::new(client.clone()),
		&task_manager.spawn_essential_handle(),
		config.prometheus_registry(),
	);

	Ok(sc_service::PartialComponents {
		client,
		backend,
		task_manager,
		import_queue,
		keystore_container,
		select_chain,
		transaction_pool,
		other: (telemetry),
	})
}

/// 构建完整客户端的新服务
/// 创建并启动完整的节点服务，包括网络、RPC、共识等所有组件
pub fn new_full<Network: sc_network::NetworkBackend<Block, <Block as BlockT>::Hash>>(
	config: Configuration,
	consensus: Consensus,
) -> Result<TaskManager, ServiceError> {
	let sc_service::PartialComponents {
		client,
		backend,
		mut task_manager,
		import_queue,
		keystore_container,
		select_chain,
		transaction_pool,
		other: mut telemetry,
	} = new_partial(&config)?;

	let net_config = sc_network::config::FullNetworkConfiguration::<
		Block,
		<Block as BlockT>::Hash,
		Network,
	>::new(
		&config.network,
		config.prometheus_config.as_ref().map(|cfg| cfg.registry.clone()),
	);
	let metrics = Network::register_notification_metrics(
		config.prometheus_config.as_ref().map(|cfg| &cfg.registry),
	);

	let (network, system_rpc_tx, tx_handler_controller, sync_service) =
		sc_service::build_network(sc_service::BuildNetworkParams {
			config: &config,
			net_config,
			client: client.clone(),
			transaction_pool: transaction_pool.clone(),
			spawn_handle: task_manager.spawn_handle(),
			spawn_essential_handle: task_manager.spawn_essential_handle(),
			import_queue,
			block_announce_validator_builder: None,
			warp_sync_config: None,
			block_relay: None,
			metrics,
		})?;

	if config.offchain_worker.enabled {
		let offchain_workers =
			sc_offchain::OffchainWorkers::new(sc_offchain::OffchainWorkerOptions {
				runtime_api_provider: client.clone(),
				is_validator: config.role.is_authority(),
				keystore: Some(keystore_container.keystore()),
				offchain_db: backend.offchain_storage(),
				transaction_pool: Some(OffchainTransactionPoolFactory::new(
					transaction_pool.clone(),
				)),
				network_provider: Arc::new(network.clone()),
				enable_http_requests: true,
				custom_extensions: |_| vec![],
			})?;
		task_manager.spawn_handle().spawn(
			"offchain-workers-runner",
			"offchain-worker",
			offchain_workers.run(client.clone(), task_manager.spawn_handle()).boxed(),
		);
	}

	let rpc_extensions_builder = {
		let client = client.clone();
		let pool = transaction_pool.clone();

		Box::new(move |_| {
			let deps = crate::rpc::FullDeps { client: client.clone(), pool: pool.clone() };
			crate::rpc::create_full(deps).map_err(Into::into)
		})
	};

	let prometheus_registry = config.prometheus_registry().cloned();

	let _rpc_handlers = sc_service::spawn_tasks(sc_service::SpawnTasksParams {
		network,
		client: client.clone(),
		keystore: keystore_container.keystore(),
		task_manager: &mut task_manager,
		transaction_pool: transaction_pool.clone(),
		rpc_builder: rpc_extensions_builder,
		backend,
		system_rpc_tx,
		tx_handler_controller,
		sync_service,
		config,
		telemetry: telemetry.as_mut(),
		tracing_execute_block: None,
	})?;

	let proposer = sc_basic_authorship::ProposerFactory::new(
		task_manager.spawn_handle(),
		client.clone(),
		transaction_pool.clone(),
		prometheus_registry.as_ref(),
		telemetry.as_ref().map(|x| x.handle()),
	);

	// 根据共识类型启动相应的出块机制
	match consensus {
		/// 即时出块：收到交易立即出块
		Consensus::InstantSeal => {
			let params = sc_consensus_manual_seal::InstantSealParams {
				block_import: client.clone(),
				env: proposer,
				client,
				pool: transaction_pool,
				select_chain,
				consensus_data_provider: None,
				create_inherent_data_providers: move |_, ()| async move {
					Ok(sp_timestamp::InherentDataProvider::from_system_time())
				},
			};

			let authorship_future = sc_consensus_manual_seal::run_instant_seal(params);

			task_manager.spawn_essential_handle().spawn_blocking(
				"instant-seal",
				None,
				authorship_future,
			);
		},
		/// 手动出块：按指定时间间隔出块
		Consensus::ManualSeal(block_time) => {
			// 创建命令通道，用于控制出块
			let (mut sink, commands_stream) = futures::channel::mpsc::channel(1024);
			// 启动定时出块任务
			task_manager.spawn_handle().spawn("block_authoring", None, async move {
				loop {
					// 等待指定的时间间隔
					futures_timer::Delay::new(std::time::Duration::from_millis(block_time)).await;
					// 发送出块命令
					sink.try_send(sc_consensus_manual_seal::EngineCommand::SealNewBlock {
						create_empty: true,  // 即使没有交易也创建空块
						finalize: true,      // 立即最终化
						parent_hash: None,   // 使用最长链的父哈希
						sender: None,        // 不需要响应通道
					})
					.unwrap();
				}
			});

			let params = sc_consensus_manual_seal::ManualSealParams {
				block_import: client.clone(),
				env: proposer,
				client,
				pool: transaction_pool,
				select_chain,
				commands_stream: Box::pin(commands_stream),
				consensus_data_provider: None,
				create_inherent_data_providers: move |_, ()| async move {
					Ok(sp_timestamp::InherentDataProvider::from_system_time())
				},
			};
			let authorship_future = sc_consensus_manual_seal::run_manual_seal(params);

			task_manager.spawn_essential_handle().spawn_blocking(
				"manual-seal",
				None,
				authorship_future,
			);
		},
		/// 无共识：不启动出块机制
		_ => {},
	}

	Ok(task_manager)
}
