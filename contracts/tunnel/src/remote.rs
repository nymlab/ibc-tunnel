use crate::error::ContractError;
use crate::state::{
    SenderInfo, ACCOUNTS, DEFAULT_LIMIT, INIT_CALLBACK_ID, MAX_LIMIT, MIGRATE_CALLBACK_ID, PENDING,
    RECEIVE_DISPATCH_ID,
};
use cosmwasm_tunnel::{
    AccountInfo, AccountResponse, DispatchMigrateResponse, InstantiateResponse,
    ListAccountsResponse, PacketMsg, QueryMsg, StdAck, WhoAmIResponse,
};

use cosmwasm_std::{
    entry_point, from_slice, to_binary, wasm_execute, CosmosMsg, Deps, DepsMut, Env, Event,
    IbcEndpoint, IbcPacketReceiveMsg, IbcQuery, IbcReceiveResponse, Order, QueryRequest,
    QueryResponse, Reply, Response, StdResult, SubMsg, WasmMsg,
};
use cw_storage_plus::Bound;
use cw_utils::parse_reply_instantiate_data;

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<QueryResponse> {
    match msg {
        QueryMsg::Account {
            connection_id,
            port_id,
            controller,
        } => to_binary(&query_account(deps, connection_id, port_id, controller)?),
        QueryMsg::ListAccounts { start_after, limit } => {
            to_binary(&query_list_accounts(deps, start_after, limit)?)
        }
    }
}

pub fn query_account(
    deps: Deps,
    connection: String,
    port: String,
    controller: String,
) -> StdResult<AccountResponse> {
    let account = ACCOUNTS.load(deps.storage, (&connection, &port, &controller))?;
    Ok(AccountResponse {
        account: Some(account.into()),
    })
}

pub fn query_list_accounts(
    deps: Deps,
    start_after: Option<(String, String, String)>,
    limit: Option<u64>,
) -> StdResult<ListAccountsResponse> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start_bound = start_after
        .as_ref()
        .map(|(c, p, s)| (c.as_str(), p.as_str(), s.as_str()))
        .map(Bound::exclusive);

    let accounts = ACCOUNTS
        .range(deps.storage, start_bound, None, Order::Ascending)
        .take(limit)
        .map(|item| {
            let ((connection, port, controller), account) = item?;
            Ok(AccountInfo {
                account: account.into(),
                connection,
                port,
                controller,
            })
        })
        .collect::<StdResult<_>>()?;
    Ok(ListAccountsResponse { accounts })
}

#[entry_point]
/// we look for a the proper reflect contract to relay to and send the message
/// We cannot return any meaningful response value as we do not know the response value
/// of execution. We just return ok if we dispatched, error if we failed to dispatch
pub fn ibc_packet_receive(
    deps: DepsMut,
    env: Env,
    msg: IbcPacketReceiveMsg,
) -> Result<IbcReceiveResponse, ContractError> {
    (|| {
        let packet = msg.packet;
        let msg: PacketMsg = from_slice(&packet.data)?;
        // The sender chain's light client id
        let connection_id = get_connection_id_from_channel(deps.as_ref(), packet.dest)?;
        // The sender's port id
        let port_id = packet.src.port_id;

        match msg {
            PacketMsg::Instantiate {
                controller,
                inst_msg,
                job_id,
                code_id,
            } => receieve_instantiate(
                deps,
                env,
                connection_id,
                port_id,
                controller,
                inst_msg,
                job_id,
                code_id,
            ),
            PacketMsg::Migrate {
                controller,
                migration_msg,
                job_id,
                new_code_id,
            } => receieve_migrate(
                deps,
                connection_id,
                port_id,
                controller,
                migration_msg,
                job_id,
                new_code_id,
            ),
            PacketMsg::Dispatch {
                msg,
                controller,
                job_id,
            } => receive_dispatch(deps, connection_id, port_id, controller, msg, job_id),
            PacketMsg::WhoAmI { controller } => {
                receive_who_am_i(deps, connection_id, port_id, controller)
            }
        }
    })()
    .or_else(|e| {
        Ok(IbcReceiveResponse::new().set_ack(StdAck::fail(format!("IBC Packet Error: {e}"))))
    })
}

#[allow(clippy::too_many_arguments)]
fn receieve_instantiate(
    deps: DepsMut,
    env: Env,
    connection_id: String,
    port_id: String,
    controller: String,
    inst_msg: CosmosMsg,
    job_id: Option<String>,
    code_id: u64,
) -> Result<IbcReceiveResponse, ContractError> {
    let msg = WasmMsg::Instantiate {
        admin: Some(env.contract.address.to_string()),
        code_id,
        msg: to_binary(&inst_msg)?,
        // TODO: allow user to deposit funds to this contract for this
        funds: vec![],
        label: format!("cosmwasm-ica-{connection_id}-{port_id}-{controller}"),
    };
    let msg = SubMsg::reply_on_success(msg, INIT_CALLBACK_ID);

    // store the relevant calling chain (host) data to be handled
    PENDING.save(
        deps.storage,
        &SenderInfo {
            connection_id,
            port_id,
            controller,
            job_id,
        },
    )?;

    Ok(IbcReceiveResponse::new()
        .add_submessage(msg)
        .add_attribute("action", "recieve_instantiate"))
}

fn receieve_migrate(
    deps: DepsMut,
    connection_id: String,
    port_id: String,
    controller: String,
    migrate_msg: CosmosMsg,
    job_id: Option<String>,
    new_code_id: u64,
) -> Result<IbcReceiveResponse, ContractError> {
    let account = ACCOUNTS.load(deps.storage, (&connection_id, &port_id, &controller))?;

    let msg = to_binary(&migrate_msg)?;
    let submsg = WasmMsg::Migrate {
        contract_addr: account.to_string(),
        new_code_id,
        msg,
    };
    let msg = SubMsg::reply_always(submsg, MIGRATE_CALLBACK_ID);

    PENDING.save(
        deps.storage,
        &SenderInfo {
            connection_id,
            port_id,
            controller,
            job_id,
        },
    )?;

    Ok(IbcReceiveResponse::new()
        .add_submessage(msg)
        .add_attribute("action", "recieve_migrate"))
}

// processes PacketMsg::Dispatch variant
fn receive_dispatch(
    deps: DepsMut,
    connection_id: String,
    port_id: String,
    controller: String,
    msg: CosmosMsg,
    job_id: Option<String>,
) -> Result<IbcReceiveResponse, ContractError> {
    let account = ACCOUNTS.load(deps.storage, (&connection_id, &port_id, &controller))?;
    let wasm_msg = wasm_execute(account, &msg, vec![])?;

    let msg = SubMsg::reply_always(wasm_msg, RECEIVE_DISPATCH_ID);

    PENDING.save(
        deps.storage,
        &SenderInfo {
            connection_id,
            port_id,
            controller,
            job_id,
        },
    )?;

    Ok(IbcReceiveResponse::new()
        .add_submessage(msg)
        .add_attribute("action", "receive_dispatch"))
}

// processes PacketMsg::WhoAmI variant
fn receive_who_am_i(
    deps: DepsMut,
    connection: String,
    port: String,
    controller: String,
) -> Result<IbcReceiveResponse, ContractError> {
    let account = ACCOUNTS.load(deps.storage, (&connection, &port, &controller))?;
    let response = WhoAmIResponse {
        account: account.into(),
    };
    let acknowledgement = StdAck::success(&response);
    Ok(IbcReceiveResponse::new()
        .set_ack(acknowledgement)
        .add_attribute("action", "receive_who_am_i"))
}

#[entry_point]
pub fn reply(deps: DepsMut, _env: Env, reply: Reply) -> Result<Response, ContractError> {
    match reply.id {
        INIT_CALLBACK_ID => reply_init_callback(deps, reply),
        RECEIVE_DISPATCH_ID => reply_dispatch_callback(deps, reply),
        MIGRATE_CALLBACK_ID => reply_migrate_callback(deps, reply),
        _ => Err(ContractError::InvalidReplyId),
    }
}

pub fn reply_init_callback(deps: DepsMut, reply: Reply) -> Result<Response, ContractError> {
    let SenderInfo {
        connection_id,
        port_id,
        controller,
        job_id,
    } = PENDING.load(deps.storage)?;
    PENDING.remove(deps.storage);

    // parse contract address from reply data
    let raw_addr = parse_reply_instantiate_data(reply)?.contract_address;
    let new_contract_addr = deps.api.addr_validate(&raw_addr)?;

    // Save this new account so it is callable in the future
    if ACCOUNTS
        .may_load(deps.storage, (&connection_id, &port_id, &controller))?
        .is_some()
    {
        return Err(ContractError::ChannelAlreadyRegistered);
    }
    ACCOUNTS.save(
        deps.storage,
        (&connection_id, &port_id, &controller),
        &new_contract_addr,
    )?;
    let event = Event::new("ica-tunnel.V1.MsgICAInstantiated").add_attributes(vec![
        ("contract_addr", new_contract_addr.to_string()),
        (
            "Controller",
            format!("{connection_id}-{port_id}-{controller}"),
        ),
    ]);

    // Send Ack to the sending chain
    let data = StdAck::success(&InstantiateResponse {
        contract_address: new_contract_addr.to_string(),
        job_id,
    });

    Ok(Response::new().set_data(data).add_event(event))
}

pub fn reply_dispatch_callback(deps: DepsMut, reply: Reply) -> Result<Response, ContractError> {
    let info = PENDING.load(deps.storage)?;
    PENDING.remove(deps.storage);
    let data = StdAck::success(&DispatchMigrateResponse {
        result: reply.result,
        job_id: info.job_id,
    });
    Ok(Response::new().set_data(data))
}

pub fn reply_migrate_callback(deps: DepsMut, reply: Reply) -> Result<Response, ContractError> {
    let info = PENDING.load(deps.storage)?;
    let data = StdAck::success(&DispatchMigrateResponse {
        result: reply.result,
        job_id: info.job_id,
    });
    Ok(Response::new().set_data(data))
}

fn get_connection_id_from_channel(
    deps: Deps,
    my_endpoint: IbcEndpoint,
) -> Result<String, ContractError> {
    use cosmwasm_std::ChannelResponse;
    let channel_resp: ChannelResponse =
        deps.querier.query(&QueryRequest::Ibc(IbcQuery::Channel {
            channel_id: my_endpoint.channel_id,
            port_id: Some(my_endpoint.port_id),
        }))?;

    Ok(channel_resp
        .channel
        .ok_or(ContractError::InvalidConnectionId)?
        .connection_id)
}
