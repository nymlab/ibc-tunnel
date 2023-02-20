pub mod contract;
pub mod error;
pub mod state;

// module used in the host chain - where the controllers live
pub mod host;
// module used in the remote chain - where the ICA or other contracts live
pub mod remote;
