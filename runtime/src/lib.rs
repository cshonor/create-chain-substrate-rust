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

//! 一个包含模板 [`pallet`](`pallet_minimal_template`) 的最小运行时
//! 
//! 这是区块链的核心逻辑，定义了所有 pallet 的配置和组合方式

#![cfg_attr(not(feature = "std"), no_std)]

// 使 WASM 二进制文件可用
// 在标准库模式下，包含编译后的 WASM 运行时二进制文件
#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

extern crate alloc;

use alloc::vec::Vec;
use pallet_transaction_payment::{FeeDetails, RuntimeDispatchInfo};
use polkadot_sdk::{
	polkadot_sdk_frame::{
		self as frame,
		deps::sp_genesis_builder,
		runtime::{apis, prelude::*},
	},
	*,
};

/// 提供创世配置预设的获取器
/// 定义不同环境的创世状态配置
pub mod genesis_config_presets {
	use super::*;
	use crate::{
		interface::{Balance, MinimumBalance},
		sp_keyring::Sr25519Keyring,
		BalancesConfig, RuntimeGenesisConfig, SudoConfig,
	};

	use alloc::{vec, vec::Vec};
	use serde_json::Value;

	/// 返回开发环境的创世配置预设
	/// 为所有测试账户预充值，并设置 Alice 为 sudo 账户
	pub fn development_config_genesis() -> Value {
		// 计算预充值金额：最小余额的 1000 倍，至少为 1
		let endowment = <MinimumBalance as Get<Balance>>::get().max(1) * 1000;
		frame_support::build_struct_json_patch!(RuntimeGenesisConfig {
			balances: BalancesConfig {
				// 为所有测试账户（Alice, Bob, Charlie 等）预充值
				balances: Sr25519Keyring::iter()
					.map(|a| (a.to_account_id(), endowment))
					.collect::<Vec<_>>(),
			},
			// 设置 Alice 为 sudo（超级管理员）账户
			sudo: SudoConfig { key: Some(Sr25519Keyring::Alice.to_account_id()) },
		})
	}

	/// Get the set of the available genesis config presets.
	pub fn get_preset(id: &PresetId) -> Option<Vec<u8>> {
		let patch = match id.as_ref() {
			sp_genesis_builder::DEV_RUNTIME_PRESET => development_config_genesis(),
			_ => return None,
		};
		Some(
			serde_json::to_string(&patch)
				.expect("serialization to json is expected to work. qed.")
				.into_bytes(),
		)
	}

	/// List of supported presets.
	pub fn preset_names() -> Vec<PresetId> {
		vec![PresetId::from(sp_genesis_builder::DEV_RUNTIME_PRESET)]
	}
}

/// 运行时版本信息
/// 用于标识和区分不同版本的运行时
#[runtime_version]
pub const VERSION: RuntimeVersion = RuntimeVersion {
	spec_name: alloc::borrow::Cow::Borrowed("minimal-template-runtime"),  // 规范名称
	impl_name: alloc::borrow::Cow::Borrowed("minimal-template-runtime"),  // 实现名称
	authoring_version: 1,      // 出块版本（影响出块者兼容性）
	spec_version: 0,           // 规范版本（影响链规范兼容性）
	impl_version: 1,           // 实现版本（用于区分实现）
	apis: RUNTIME_API_VERSIONS, // 运行时 API 版本
	transaction_version: 1,     // 交易版本（影响交易格式）
	system_version: 1,         // 系统版本
};

/// The version information used to identify this runtime when compiled natively.
#[cfg(feature = "std")]
pub fn native_version() -> NativeVersion {
	NativeVersion { runtime_version: VERSION, can_author_with: Default::default() }
}

/// 运行时添加的交易扩展
/// 定义交易在执行前需要经过的所有检查和操作（按顺序执行）
type TxExtension = (
	// 授权调用，验证调用本身的有效性
	frame_system::AuthorizeCall<Runtime>,
	// 检查发送者不是零地址
	frame_system::CheckNonZeroSender<Runtime>,
	// 检查运行时版本是否正确
	frame_system::CheckSpecVersion<Runtime>,
	// 检查交易版本是否正确
	frame_system::CheckTxVersion<Runtime>,
	// 检查创世哈希是否正确
	frame_system::CheckGenesis<Runtime>,
	// 检查 era（时代）是否有效
	frame_system::CheckEra<Runtime>,
	// 检查 nonce（交易序号）是否有效
	frame_system::CheckNonce<Runtime>,
	// 检查权重是否有效
	frame_system::CheckWeight<Runtime>,
	// 确保发送者有足够的资金支付交易费用，并从发送者账户扣除费用
	pallet_transaction_payment::ChargeTransactionPayment<Runtime>,
	// 使用调度后信息回收区块中未使用的权重
	// 必须在管道的最后，以便捕获之前交易扩展中的退款
	frame_system::WeightReclaim<Runtime>,
);

// 通过添加所有使用的 pallet 并派生必要的类型来组合运行时
#[frame_construct_runtime]
mod runtime {
	/// 主运行时类型
	/// 这是整个运行时的核心结构，包含所有 pallet 的配置
	#[runtime::runtime]
	#[runtime::derive(
		RuntimeCall,           // 运行时调用类型
		RuntimeEvent,          // 运行时事件类型
		RuntimeError,          // 运行时错误类型
		RuntimeOrigin,         // 运行时来源类型
		RuntimeFreezeReason,  // 冻结原因类型
		RuntimeHoldReason,     // 保留原因类型
		RuntimeSlashReason,    // 惩罚原因类型
		RuntimeLockId,         // 锁定 ID 类型
		RuntimeTask,           // 运行时任务类型
		RuntimeViewFunction    // 运行时视图函数类型
	)]
	pub struct Runtime;

	/// 系统 pallet（索引 0）
	/// 必须在 FRAME 运行时中包含的强制性系统 pallet，提供基础功能
	#[runtime::pallet_index(0)]
	pub type System = frame_system::Pallet<Runtime>;

	/// 时间戳 pallet（索引 1）
	/// 为共识系统提供设置和检查链上时间的方式
	#[runtime::pallet_index(1)]
	pub type Timestamp = pallet_timestamp::Pallet<Runtime>;

	/// 余额 pallet（索引 2）
	/// 提供跟踪账户余额的能力
	#[runtime::pallet_index(2)]
	pub type Balances = pallet_balances::Pallet<Runtime>;

	/// Sudo pallet（索引 3）
	/// 提供执行特权函数的方式（超级管理员功能）
	#[runtime::pallet_index(3)]
	pub type Sudo = pallet_sudo::Pallet<Runtime>;

	/// 交易支付 pallet（索引 4）
	/// 提供对外部调用执行收费的能力
	#[runtime::pallet_index(4)]
	pub type TransactionPayment = pallet_transaction_payment::Pallet<Runtime>;

	/// 最小模板 pallet（索引 5）
	/// 一个最小化的 pallet 模板，作为自定义 pallet 的起点
	#[runtime::pallet_index(5)]
	pub type Template = pallet_minimal_template::Pallet<Runtime>;
}

parameter_types! {
	pub const Version: RuntimeVersion = VERSION;
}

/// 实现系统 pallet 所需的类型
/// 配置系统 pallet 的基本参数
#[derive_impl(frame_system::config_preludes::SolochainDefaultConfig)]
impl frame_system::Config for Runtime {
	type Block = Block;           // 区块类型
	type Version = Version;       // 版本信息
	// 使用余额 pallet 的账户数据
	type AccountData = pallet_balances::AccountData<<Runtime as pallet_balances::Config>::Balance>;
}

// 实现余额 pallet 所需的类型
#[derive_impl(pallet_balances::config_preludes::TestDefaultConfig)]
impl pallet_balances::Config for Runtime {
	type AccountStore = System;  // 使用系统 pallet 存储账户数据
}

// 实现 sudo pallet 所需的类型
#[derive_impl(pallet_sudo::config_preludes::TestDefaultConfig)]
impl pallet_sudo::Config for Runtime {}

// 实现时间戳 pallet 所需的类型
#[derive_impl(pallet_timestamp::config_preludes::TestDefaultConfig)]
impl pallet_timestamp::Config for Runtime {}

// 实现交易支付 pallet 所需的类型
#[derive_impl(pallet_transaction_payment::config_preludes::TestDefaultConfig)]
impl pallet_transaction_payment::Config for Runtime {
	type OnChargeTransaction = pallet_transaction_payment::FungibleAdapter<Balances, ()>;
	// 为了演示目的，将费用设置为与外部调用的权重无关（免费）
	type WeightToFee = NoFee<<Self as pallet_balances::Config>::Balance>;
	// 为了演示目的，将费用设置为固定值，与调用数据长度无关
	type LengthToFee = FixedFee<1, <Self as pallet_balances::Config>::Balance>;
}

// 实现模板 pallet 所需的类型
impl pallet_minimal_template::Config for Runtime {}

type Block = frame::runtime::types_common::BlockOf<Runtime, TxExtension>;
type Header = HeaderFor<Runtime>;

type RuntimeExecutive =
	Executive<Runtime, Block, frame_system::ChainContext<Runtime>, Runtime, AllPalletsWithSystem>;

impl_runtime_apis! {
	impl apis::Core<Block> for Runtime {
		fn version() -> RuntimeVersion {
			VERSION
		}

		fn execute_block(block: <Block as frame::traits::Block>::LazyBlock) {
			RuntimeExecutive::execute_block(block)
		}

		fn initialize_block(header: &Header) -> ExtrinsicInclusionMode {
			RuntimeExecutive::initialize_block(header)
		}
	}
	impl apis::Metadata<Block> for Runtime {
		fn metadata() -> OpaqueMetadata {
			OpaqueMetadata::new(Runtime::metadata().into())
		}

		fn metadata_at_version(version: u32) -> Option<OpaqueMetadata> {
			Runtime::metadata_at_version(version)
		}

		fn metadata_versions() -> Vec<u32> {
			Runtime::metadata_versions()
		}
	}

	impl apis::BlockBuilder<Block> for Runtime {
		fn apply_extrinsic(extrinsic: ExtrinsicFor<Runtime>) -> ApplyExtrinsicResult {
			RuntimeExecutive::apply_extrinsic(extrinsic)
		}

		fn finalize_block() -> HeaderFor<Runtime> {
			RuntimeExecutive::finalize_block()
		}

		fn inherent_extrinsics(data: InherentData) -> Vec<ExtrinsicFor<Runtime>> {
			data.create_extrinsics()
		}

		fn check_inherents(
			block: <Block as frame::traits::Block>::LazyBlock,
			data: InherentData,
		) -> CheckInherentsResult {
			data.check_extrinsics(&block)
		}
	}

	impl apis::TaggedTransactionQueue<Block> for Runtime {
		fn validate_transaction(
			source: TransactionSource,
			tx: ExtrinsicFor<Runtime>,
			block_hash: <Runtime as frame_system::Config>::Hash,
		) -> TransactionValidity {
			RuntimeExecutive::validate_transaction(source, tx, block_hash)
		}
	}

	impl apis::OffchainWorkerApi<Block> for Runtime {
		fn offchain_worker(header: &HeaderFor<Runtime>) {
			RuntimeExecutive::offchain_worker(header)
		}
	}

	impl apis::SessionKeys<Block> for Runtime {
		fn generate_session_keys(_owner: Vec<u8>, _seed: Option<Vec<u8>>) -> apis::OpaqueGeneratedSessionKeys {
			apis::OpaqueGeneratedSessionKeys { keys: Default::default(), proof: Default::default() }
		}

		fn decode_session_keys(
			_encoded: Vec<u8>,
		) -> Option<Vec<(Vec<u8>, apis::KeyTypeId)>> {
			Default::default()
		}
	}

	impl apis::AccountNonceApi<Block, interface::AccountId, interface::Nonce> for Runtime {
		fn account_nonce(account: interface::AccountId) -> interface::Nonce {
			System::account_nonce(account)
		}
	}

	impl pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi<
		Block,
		interface::Balance,
	> for Runtime {
		fn query_info(uxt: ExtrinsicFor<Runtime>, len: u32) -> RuntimeDispatchInfo<interface::Balance> {
			TransactionPayment::query_info(uxt, len)
		}
		fn query_fee_details(uxt: ExtrinsicFor<Runtime>, len: u32) -> FeeDetails<interface::Balance> {
			TransactionPayment::query_fee_details(uxt, len)
		}
		fn query_weight_to_fee(weight: Weight) -> interface::Balance {
			TransactionPayment::weight_to_fee(weight)
		}
		fn query_length_to_fee(length: u32) -> interface::Balance {
			TransactionPayment::length_to_fee(length)
		}
	}

	impl apis::GenesisBuilder<Block> for Runtime {
		fn build_state(config: Vec<u8>) -> sp_genesis_builder::Result {
			build_state::<RuntimeGenesisConfig>(config)
		}

		fn get_preset(id: &Option<PresetId>) -> Option<Vec<u8>> {
			get_preset::<RuntimeGenesisConfig>(id, self::genesis_config_presets::get_preset)
		}

		fn preset_names() -> Vec<PresetId> {
			self::genesis_config_presets::preset_names()
		}
	}
}

/// Some re-exports that the node side code needs to know. Some are useful in this context as well.
///
/// Other types should preferably be private.
// TODO: this should be standardized in some way, see:
// https://github.com/paritytech/substrate/issues/10579#issuecomment-1600537558
pub mod interface {
	use super::Runtime;
	use polkadot_sdk::{polkadot_sdk_frame as frame, *};

	pub type Block = super::Block;
	pub use frame::runtime::types_common::OpaqueBlock;
	pub type AccountId = <Runtime as frame_system::Config>::AccountId;
	pub type Nonce = <Runtime as frame_system::Config>::Nonce;
	pub type Hash = <Runtime as frame_system::Config>::Hash;
	pub type Balance = <Runtime as pallet_balances::Config>::Balance;
	pub type MinimumBalance = <Runtime as pallet_balances::Config>::ExistentialDeposit;
}
