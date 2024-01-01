//! The `lib_rpc::response` module normalizes the JSON-RPC `.result` format for various
//! JSON-RPC APIs.
//!
//! The primary type is the simple DataRpcResult, which contains only a `data` property.
//!
//! Notes:
//!   - In the future, we may introduce types like `DataRpcResult` that include metadata
//!     about the returned list data (e.g., pagination information).
//!   - Although the struct is named with `Result`, it is not a typical Rust result. Instead,
//!     it represents the `.result` property of a JSON-RPC response.
//!

use serde::Serialize;

#[derive(Serialize)]
pub struct DataRpcResult<T>
where
	T: Serialize,
{
	data: T,
}

impl<T> From<T> for DataRpcResult<T>
where
	T: Serialize,
{
	fn from(val: T) -> Self {
		Self { data: val }
	}
}
