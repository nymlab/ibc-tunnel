use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};

#[cw_serde]
pub struct SenderInfo {
    /// Sender's light client id on this chain
    pub connection_id: String,
    /// Sender's module / cw contract id
    pub port_id: String,
    /// Sender's account on the remote chain
    pub controller: String,
    /// Job id for ref on the sending chain
    pub job_id: Option<String>,
}

// Pending storage data structure for any operations, removed on success, replace by new call on failure
// Mainly used to pass IBC information to be handled in `reply`
pub const PENDING: Item<SenderInfo> = Item::new("pending");
// We map the ibc endpoint (trusted) and the relayed sender (untrusted) here
pub const ACCOUNTS: Map<(&str, &str, &str), Addr> = Map::new("accounts");

pub const RECEIVE_DISPATCH_ID: u64 = 1234;
pub const INIT_CALLBACK_ID: u64 = 7890;
pub const MIGRATE_CALLBACK_ID: u64 = 7899;
pub const DEFAULT_LIMIT: u64 = 50;
pub const MAX_LIMIT: u64 = 300;
