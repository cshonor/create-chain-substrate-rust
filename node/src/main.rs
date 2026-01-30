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

//! Substrate Node Template CLI library.
//! Substrate 节点模板 CLI 库
#![warn(missing_docs)]

mod chain_spec;  // 链规范配置模块
mod cli;         // 命令行接口模块
mod command;     // 命令处理模块
mod rpc;         // RPC 接口模块
mod service;     // 服务构建模块

/// 程序入口点
/// 解析命令行参数并执行相应的命令
fn main() -> polkadot_sdk::sc_cli::Result<()> {
	command::run()
}
