# Ibc Tunnels

The purpose of this project is to make it easier for other developers to instantiate, migrate and dispatch message across different cosmos chains.
The [tunnel](.contracts/tunnel) is used as both the host (controller of the remote proxy contract) and the remote ( reciever of instructions from the controller on the host chain).

This is inspired by the "Simple ICA" contracts that is the base of this project. [Ethan's HackAtom Video](https://www.youtube.com/watch?v=x75UobIr4qo&t=9070s).
This is also an extension of the work on [cosmwasm-ica](https://github.com/j0nl1/cosmwasm-ica).

[cosmwasm-tunnel](https://crates.io/crates/cosmwasm-tunnel) is a library to facilitate the use of IBC tunnels in your contract for sending/receiving message to/from the host.

## Motivation

In [Vectis], the VectisDAO treats wallet from different chains the same, and therefore requires tunnel to forward their transactions with the DAO.
We have [remote-tunnel] contract for all contracts that are not deployed on Juno, which uses [dao-tunnel].

In order to ensure all chains [Vectis] deploys on can have upgradable [remote-tunnel] contracts, a simple instantiation, migration and dispatch tunnel is required.
On the host chain - Juno, the DAO can upgrade the [dao-tunnel]

## Design outline

The most simple operations can be done here (Until we implement a way to recieve payment for operations to relay and forward funds).

### Roles

- **Controller**: This is the role on the host (controlling) chain
- **Proxy**: This is a contract instantiated by the **Controller** on a remote chain. The **Controller** can then dispatch messages through this **Proxy** and also migrate it. The most obviously use case for this is Interchain Account wher the **Proxy** is a [cw1-whitelist] contract on th remote chain.

### Remote Instantiate

The **Controller** on the host chain can instantiate any **Proxy** contract on the remote chain by passing in `code_id` and `InstantiateMsg`.
This **Proxy** address will then be store in the tunnel as controller by the combination of:

1. connection-id: The underlying light client of the remote chain
2. port-id: The calling module to the tunnel contract
3. controller: specified as the `info.sender` of the `ExecuteMsg::RemoteInstantiate` message on the host chain.

IBC is permissionless, `channel-id` is incremental and can be considered the route the message came from, but not the source.

### Remote Migrate

This allows the **Controller** to migrate their **Proxy** contract to another version by passing in `MigrateMsg` and the `new_code_id`.

### Remote Dispatch

This allows the **Controller** to dispatch a message through their **Proxy** contract.
For example, if this the **Proxy** is an Interchain Account cw1-whitelist, then this can dispatch the [ExecuteMsg::Execute] message on the interchain account.
The tunnel contract will find the **Proxy** address.

## Deployed on

| Chain | Network | Contract Address | Ibc tunnel Code ID | Tunnel Upload Tx & Instantiation Tx |
| ----- | ------- | ---------------- | ------------------ | ----------------------------------- |
| Juno  | Testnet |                  |                    |                                     |

[cw1-whitelist]: https://github.com/CosmWasm/cw-plus/tree/main/contracts/cw1-whitelist
[vectis]: https://github.com/nymlab/vectis
[dao-tunnel]: https://github.com/nymlab/vectis/tree/main/contracts/dao_tunnel
[remote-tunnel]: https://github.com/nymlab/vectis/tree/main/contracts/remote_tunnel
[executemsg::execute]: https://github.com/CosmWasm/cw-plus/blob/main/contracts/cw1-whitelist/src/msg.rs#L61
