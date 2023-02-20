mod checks;
mod msg;

use cosmwasm_std::IbcOrder;

pub use crate::checks::{check_order, check_version, ChannelError};
pub use crate::msg::*;

pub const IBC_APP_VERSION: &str = "cw-tunnel-v1";
pub const APP_ORDER: IbcOrder = IbcOrder::Unordered;
pub const BAD_APP_ORDER: IbcOrder = IbcOrder::Ordered;
pub const PACKET_LIFETIME: u64 = 60 * 60;
