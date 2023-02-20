use crate::error::ContractError;
use cosmwasm_tunnel::{
    ExecuteMsg, PacketMsg, PACKET_LIFETIME,
};

use cosmwasm_std::{
    entry_point, to_binary, CosmosMsg, DepsMut, Env, Event, IbcBasicResponse, IbcMsg, IbcPacketAckMsg, IbcPacketTimeoutMsg, MessageInfo,
    Response, StdResult,
};

#[entry_point]
pub fn execute(
    _deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::RemoteInstantiate {
            inst_msg,
            code_id,
            job_id,
            channel_id,
        } => execute_remote_instantiate(info, env, inst_msg, code_id, job_id, channel_id),
        ExecuteMsg::RemoteMigrate {
            migrate_msg,
            job_id,
            new_code_id,
            channel_id,
        } => execute_remote_migrate(info, env, migrate_msg, new_code_id, job_id, channel_id),
        ExecuteMsg::RemoteDispatch {
            dispatch_msg,
            job_id,
            channel_id,
        } => execute_remote_dispatch(info, env, dispatch_msg, job_id, channel_id),
        ExecuteMsg::QueryRemoteAddr { channel_id } => execute_remote_query(info, env, channel_id),
    }
}

fn create_ibc_msg(channel_id: String, data: PacketMsg, env: Env) -> Result<IbcMsg, ContractError> {
    Ok(IbcMsg::SendPacket {
        channel_id,
        data: to_binary(&data)?,
        timeout: env.block.time.plus_seconds(PACKET_LIFETIME).into(),
    })
}

pub fn execute_remote_instantiate(
    info: MessageInfo,
    env: Env,
    inst_msg: CosmosMsg,
    code_id: u64,
    job_id: Option<String>,
    channel_id: String,
) -> Result<Response, ContractError> {
    let packet: PacketMsg = PacketMsg::Instantiate {
        controller: info.sender.to_string(),
        inst_msg,
        code_id,
        job_id: job_id.clone(),
    };
    let msg = create_ibc_msg(channel_id.clone(), packet, env)?;

    let event = Event::new("ica-tunnel.V1.HostMsg.InstantiationRequested")
        .add_attribute("channel_id", channel_id)
        .add_attribute("job_id", format!("{job_id:?}"));

    Ok(Response::new().add_message(msg).add_event(event))
}

pub fn execute_remote_migrate(
    info: MessageInfo,
    env: Env,
    migration_msg: CosmosMsg,
    new_code_id: u64,
    job_id: Option<String>,
    channel_id: String,
) -> Result<Response, ContractError> {
    let packet: PacketMsg = PacketMsg::Migrate {
        controller: info.sender.to_string(),
        migration_msg,
        job_id: job_id.clone(),
        new_code_id,
    };
    let msg = create_ibc_msg(channel_id.clone(), packet, env)?;

    let event = Event::new("ica-tunnel.V1.HostMsg.MigrationRequested")
        .add_attribute("channel_id", channel_id)
        .add_attribute("job_id", format!("{job_id:?}"));

    Ok(Response::new().add_message(msg).add_event(event))
}

pub fn execute_remote_dispatch(
    info: MessageInfo,
    env: Env,
    msg: CosmosMsg,
    job_id: Option<String>,
    channel_id: String,
) -> Result<Response, ContractError> {
    let packet: PacketMsg = PacketMsg::Dispatch {
        controller: info.sender.to_string(),
        msg,
        job_id: job_id.clone(),
    };
    let msg = create_ibc_msg(channel_id.clone(), packet, env)?;

    let event = Event::new("ica-tunnel.V1.HostMsg.DispatchRequested")
        .add_attribute("channel_id", channel_id)
        .add_attribute("job_id", format!("{job_id:?}"));

    Ok(Response::new().add_message(msg).add_event(event))
}

pub fn execute_remote_query(
    info: MessageInfo,
    env: Env,
    channel_id: String,
) -> Result<Response, ContractError> {
    let packet: PacketMsg = PacketMsg::WhoAmI {
        controller: info.sender.to_string(),
    };
    let msg = create_ibc_msg(channel_id.clone(), packet, env)?;

    let event = Event::new("ica-tunnel.V1.HostMsg.RemoteAddrRequested")
        .add_attribute("channel_id", channel_id)
        .add_attribute("controller", info.sender);

    Ok(Response::new().add_message(msg).add_event(event))
}

#[entry_point]
pub fn ibc_packet_ack(
    _deps: DepsMut,
    _env: Env,
    _msg: IbcPacketAckMsg,
) -> StdResult<IbcBasicResponse> {
    Ok(IbcBasicResponse::new().add_attribute("action", "ibc_packet_ack"))
}

#[entry_point]
pub fn ibc_packet_timeout(
    _deps: DepsMut,
    _env: Env,
    _msg: IbcPacketTimeoutMsg,
) -> StdResult<IbcBasicResponse> {
    Ok(IbcBasicResponse::new().add_attribute("action", "ibc_packet_timeout"))
}
