use crate::router::{Error, Result};
use serde::de::DeserializeOwned;
use serde_json::Value;

/// `IntoHandlerParams` allows for converting an `Option<Value>` into
/// the necessary type for RPC handler parameters.
/// The default implementation below will result in failure if the value is `None`.
/// For customized behavior, users can implement their own `into_handler_params`
/// method.
pub trait IntoParams: DeserializeOwned + Send {
	fn into_params(value: Option<Value>) -> Result<Self> {
		match value {
			Some(value) => Ok(serde_json::from_value(value)?),
			None => Err(Error::RpcIntoParamsMissing),
		}
	}
}

/// Marker trait with a blanket implementation that return T::default
/// if the `params: Option<Value>` is none.
pub trait IntoDefaultParams: DeserializeOwned + Send + Default {}

impl<P> IntoParams for P
where
	P: IntoDefaultParams,
{
	fn into_params(value: Option<Value>) -> Result<Self> {
		match value {
			Some(value) => Ok(serde_json::from_value(value)?),
			None => Ok(Self::default()),
		}
	}
}
