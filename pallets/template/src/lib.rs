//! 使用 [`frame`] 构建的模板 pallet
//!
//! 这是一个最小化的 pallet 模板，作为创建自定义 pallet 的起点
//! 
//! 要开始使用此 pallet，请尝试实现以下指南：
//! <https://paritytech.github.io/polkadot-sdk/master/polkadot_sdk_docs/guides/your_first_pallet/index.html>

#![cfg_attr(not(feature = "std"), no_std)]

use frame::prelude::*;
use polkadot_sdk::polkadot_sdk_frame as frame;

// 重新导出所有 pallet 部分，这是将 pallet 正确导入运行时所需的
pub use pallet::*;

#[frame::pallet]
pub mod pallet {
	use super::*;

	/// Pallet 配置 trait
	/// 定义此 pallet 所需的配置类型
	#[pallet::config]
	pub trait Config: polkadot_sdk::frame_system::Config {}

	/// Pallet 结构体
	/// 这是 pallet 的主要结构，使用 `()` 表示不需要存储任何数据
	#[pallet::pallet]
	pub struct Pallet<T>(_);

	/// 存储值
	/// 存储一个 u32 类型的值
	#[pallet::storage]
	pub type Value<T> = StorageValue<Value = u32>;
}
