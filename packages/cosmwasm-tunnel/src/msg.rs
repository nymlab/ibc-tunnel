use cosmwasm_schema::{cw_serde, serde, QueryResponses};
use cosmwasm_std::{from_slice, to_binary, Binary, CosmosMsg, SubMsgResult};

#[cw_serde]
pub struct InstantiateMsg {
    pub init: u64,
}

#[cw_serde]
pub enum ExecuteMsg {
    RemoteInstantiate {
        inst_msg: CosmosMsg,
        job_id: Option<String>,
        code_id: u64,
        channel_id: String,
    },
    RemoteMigrate {
        migrate_msg: CosmosMsg,
        job_id: Option<String>,
        new_code_id: u64,
        channel_id: String,
    },
    RemoteDispatch {
        dispatch_msg: CosmosMsg,
        job_id: Option<String>,

        channel_id: String,
    },
    QueryRemoteAddr {
        channel_id: String,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// Returns (reflect) account that is attached to this channel,
    /// or none.
    #[returns(AccountResponse)]
    Account {
        connection_id: String,
        port_id: String,
        // This controller account can migrate the ICA
        controller: String,
    },
    #[returns(ListAccountsResponse)]
    ListAccounts {
        /// pagination (connection-id, port-id, controller)
        start_after: Option<(String, String, String)>,
        limit: Option<u64>,
    },
}

#[cw_serde]
pub struct AccountResponse {
    pub account: Option<String>,
}

#[cw_serde]
pub struct ListAccountsResponse {
    pub accounts: Vec<AccountInfo>,
}

#[cw_serde]
pub struct AccountInfo {
    pub account: String,
    pub connection: String,
    pub port: String,
    pub controller: String,
}

/// This is the message we send over the IBC channel
#[cw_serde]
pub enum PacketMsg {
    Instantiate {
        controller: String,
        inst_msg: CosmosMsg,
        job_id: Option<String>,
        code_id: u64,
    },
    Migrate {
        controller: String,
        migration_msg: CosmosMsg,
        new_code_id: u64,
        job_id: Option<String>,
    },
    Dispatch {
        controller: String,
        msg: CosmosMsg,
        job_id: Option<String>,
    },
    WhoAmI {
        controller: String,
    },
}

/// This is a generic ICS acknowledgement format.
/// Proto defined here: https://github.com/cosmos/cosmos-sdk/blob/v0.42.0/proto/ibc/core/channel/v1/channel.proto#L141-L147
/// If ibc_receive_packet returns Err(), then x/wasm runtime will rollback the state and return an error message in this format
#[cw_serde]
pub enum StdAck {
    Result(Binary),
    Error(String),
}

impl StdAck {
    // create a serialized success message
    pub fn success(data: impl serde::Serialize) -> Binary {
        let res = to_binary(&data).unwrap();
        StdAck::Result(res).ack()
    }

    // create a serialized error message
    pub fn fail(err: String) -> Binary {
        StdAck::Error(err).ack()
    }

    pub fn ack(&self) -> Binary {
        to_binary(self).unwrap()
    }

    pub fn unwrap(self) -> Binary {
        match self {
            StdAck::Result(data) => data,
            StdAck::Error(err) => panic!("{}", err),
        }
    }

    pub fn unwrap_into<T: serde::de::DeserializeOwned>(self) -> T {
        from_slice(&self.unwrap()).unwrap()
    }

    pub fn unwrap_err(self) -> String {
        match self {
            StdAck::Result(_) => panic!("not an error"),
            StdAck::Error(err) => err,
        }
    }
}

/// Return the data field for each message
#[cw_serde]
pub struct InstantiateResponse {
    pub contract_address: String,
    pub job_id: Option<String>,
}

/// Return the data field for each message
#[cw_serde]
pub struct DispatchMigrateResponse {
    pub result: SubMsgResult,
    pub job_id: Option<String>,
}

/// This is the success response we send on ack for PacketMsg::WhoAmI.
/// Return the caller's account address on the remote chain
#[cw_serde]
pub struct WhoAmIResponse {
    pub account: String,
}
