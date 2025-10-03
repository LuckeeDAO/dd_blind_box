//! dd_blind_box
//!
//! A CosmWasm blind box contract supporting:
//! - Fixed scale NFT supply (10/100/1k/10k/100k)
//! - Base-coin multiple deposit → sequential NFT distribution
//! - Owner-settable base coin and vote state
//! - Commit–reveal voting with sha256(addr|reveal|salt) verification
//! - Tiered settlement using dd_algorithms_lib (10%/50%/40%)
//!
//! CosmWasm 盲盒合约，功能包括：
//! - 固定规模的 NFT 铸造供应（10/100/1000/10000/100000）
//! - 基础代币按倍数充值，按顺序分配 NFT（从 0 递增的 token_id）
//! - 拥有者可设置基础代币与投票阶段（提交/揭示/关闭）
//! - 提交-揭示式投票：验证规则为 sha256(addr|reveal|salt)
//! - 使用 dd_algorithms_lib 进行分层结算（10%/50%/40%）
pub mod contract;
pub mod error;
pub mod msg;
pub mod state;

pub use crate::error::ContractError;
pub use crate::contract::{instantiate, execute, query, migrate, reply};

#[cfg(test)]
mod tests;


