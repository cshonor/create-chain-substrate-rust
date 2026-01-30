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

use minimal_template_runtime::WASM_BINARY;
use polkadot_sdk::{
	sc_service::{ChainType, Properties},
	*,
};

/// 链规范类型别名
/// 这是通用 Substrate ChainSpec 类型的特化版本
pub type ChainSpec = sc_service::GenericChainSpec;

/// 获取链的属性配置
/// 包括代币小数位数和代币符号
fn props() -> Properties {
	let mut properties = Properties::new();
	properties.insert("tokenDecimals".to_string(), 0.into());  // 代币小数位数
	properties.insert("tokenSymbol".to_string(), "MINI".into()); // 代币符号
	properties
}

/// 创建开发链规范
/// 用于本地开发和测试的链配置
pub fn development_chain_spec() -> Result<ChainSpec, String> {
	Ok(ChainSpec::builder(WASM_BINARY.expect("Development wasm not available"), Default::default())
		.with_name("Development")  // 链名称
		.with_id("dev")            // 链 ID
		.with_chain_type(ChainType::Development)  // 链类型：开发链
		.with_genesis_config_preset_name(sp_genesis_builder::DEV_RUNTIME_PRESET)  // 创世配置预设
		.with_properties(props())  // 链属性
		.build())
}
