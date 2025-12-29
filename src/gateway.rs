//! Gateway module - 通道运行时抽象层
//!
//! 本模块提供：
//! - `ChannelRuntime` trait：统一的协议通道接口
//! - `wrappers`：各协议的 ChannelRuntime 实现
//! - `factory`：根据配置创建通道的工厂函数
//! - 配置类型和地址解析
//!
//! # 架构定位
//!
//! igw 是**纯协议库**，只提供"积木"（协议适配器 + 统一接口）。
//! 调度循环、事件处理、存储等业务逻辑由使用方（如 comsrv）实现。
//!
//! # 使用示例
//!
//! ```rust,ignore
//! use igw::gateway::{factory, ChannelConfig, ChannelRuntime};
//!
//! // 创建通道
//! let config = ChannelConfig { ... };
//! let mut channel: Box<dyn ChannelRuntime> = factory::create_channel(&config)?;
//!
//! // 连接
//! channel.connect().await?;
//!
//! // 轮询（用户自己实现调度循环）
//! loop {
//!     let result = channel.poll_once().await;
//!     // 处理 result.data 和 result.failures
//! }
//!
//! // 断开
//! channel.disconnect().await?;
//! ```
//!
//! 完整的网关运行时示例见 `examples/gateway_demo.rs`。

// Submodules in gateway/ directory
#[path = "gateway/address.rs"]
mod address;
#[path = "gateway/config.rs"]
mod config;
#[path = "gateway/factory.rs"]
pub mod factory;
#[path = "gateway/runtime.rs"]
mod runtime;
#[path = "gateway/wrappers.rs"]
pub mod wrappers;

// Public exports
pub use address::parse_address;
pub use config::{
    ChannelConfig, ChannelModeConfig, ConfigError, GatewayConfig, GatewayGlobalConfig, PointDef,
};
pub use runtime::{ChannelMode, ChannelRuntime};
