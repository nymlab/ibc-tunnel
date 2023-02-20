use crate::error::ContractError;
use cosmwasm_tunnel::{check_order, check_version, IBC_APP_VERSION};

use cosmwasm_std::{
    entry_point, DepsMut, Env, Event, Ibc3ChannelOpenResponse, IbcBasicResponse,
    IbcChannelCloseMsg, IbcChannelConnectMsg, IbcChannelOpenMsg, IbcChannelOpenResponse,
    MessageInfo, Response, StdResult,
};

#[entry_point]
pub fn instantiate(_deps: DepsMut, _env: Env, _info: MessageInfo) -> StdResult<Response> {
    let event = Event::new("ica-tunnel.V1.MsgInstantiated");
    Ok(Response::new().add_event(event))
}

#[entry_point]
/// enforces ordering and versioing constraints
pub fn ibc_channel_open(
    _deps: DepsMut,
    _env: Env,
    msg: IbcChannelOpenMsg,
) -> Result<IbcChannelOpenResponse, ContractError> {
    let channel = msg.channel();

    check_order(&channel.order)?;
    // In ibcv3 we don't check the version string passed in the message
    // and only check the counterparty version.
    if let Some(counter_version) = msg.counterparty_version() {
        check_version(counter_version)?;
    }

    // We return the version we need (which could be different than the counterparty version)
    Ok(Some(Ibc3ChannelOpenResponse {
        version: IBC_APP_VERSION.to_string(),
    }))
}

#[entry_point]
/// On connect, we do not do anything other than just logging the information
pub fn ibc_channel_connect(
    _deps: DepsMut,
    _env: Env,
    msg: IbcChannelConnectMsg,
) -> StdResult<IbcBasicResponse> {
    // We do not use channel here because the channels can be closed permisionlessly
    // connection_id: this is the id for the light client on the counterparty chain
    // port_id: this is the counterparty module / wasm smart contract

    Ok(IbcBasicResponse::new()
        .add_attribute("action", "ibc channel connect")
        .add_attribute("counterparty client id", &msg.channel().connection_id)
        .add_attribute(
            "counterparty port",
            &msg.channel().counterparty_endpoint.port_id,
        )
        .add_attribute("current channel", &msg.channel().endpoint.channel_id)
        .add_event(Event::new("ibc").add_attribute("channel", "connect")))
}

#[entry_point]
/// On closed channel, we do not need to do anything other than logging the channel id being
/// closed.
pub fn ibc_channel_close(
    _deps: DepsMut,
    _env: Env,
    msg: IbcChannelCloseMsg,
) -> StdResult<IbcBasicResponse> {
    Ok(IbcBasicResponse::new()
        .add_attribute("action", "ibc channel close ")
        .add_attribute("counterparty client id", &msg.channel().connection_id)
        .add_attribute(
            "counterparty port",
            &msg.channel().counterparty_endpoint.port_id,
        )
        .add_attribute("current channel", &msg.channel().endpoint.channel_id)
        .add_event(Event::new("ibc").add_attribute("channel", "close")))
}
