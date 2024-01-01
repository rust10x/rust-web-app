// region:    --- Modules

mod error;
mod resources;
mod rpc_params;
mod rpc_result;
mod rpcs;

pub mod router;

pub use self::error::{Error, Result};
pub use resources::RpcResources;
pub use router::RpcRequest;
pub use rpc_params::*;

pub use rpcs::*;

// endregion: --- Modules
