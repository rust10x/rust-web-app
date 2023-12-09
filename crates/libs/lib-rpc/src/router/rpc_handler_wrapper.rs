use crate::router::Result;
use crate::router::{PinFutureValue, RpcHandler};
use crate::RpcResources;
use futures::Future;
use serde_json::Value;
use std::marker::PhantomData;
use std::pin::Pin;

/// `RpcHanlderWrapper` is a `RpcHandler` wrapper which implements
/// `RpcHandlerWrapperTrait` for type erasure, enabling dynamic dispatch.
#[derive(Clone)]
pub struct RpcHandlerWrapper<H, T, P, R> {
	handler: H,
	_marker: PhantomData<(T, P, R)>,
}

// Constructor
impl<H, T, P, R> RpcHandlerWrapper<H, T, P, R> {
	pub fn new(handler: H) -> Self {
		Self {
			handler,
			_marker: PhantomData,
		}
	}
}

// Call Impl
impl<H, T, P, R> RpcHandlerWrapper<H, T, P, R>
where
	H: RpcHandler<T, P, R> + Send + Sync + 'static,
	T: Send + Sync + 'static,
	P: Send + Sync + 'static,
	R: Send + Sync + 'static,
{
	pub fn call(
		&self,
		rpc_resources: RpcResources,
		params: Option<Value>,
	) -> H::Future {
		// Note: Since handler is a FnOnce, we can use it only once, so we clone it.
		//       This is likely optimized by the compiler.
		let handler = self.handler.clone();
		RpcHandler::call(handler, rpc_resources, params)
	}
}

/// `RpcHandlerWrapperTrait` enables `RpcHandlerWrapper` to become a trait object,
/// allowing for dynamic dispatch.
pub trait RpcHandlerWrapperTrait: Send + Sync {
	fn call(
		&self,
		rpc_resources: RpcResources,
		params: Option<Value>,
	) -> PinFutureValue;
}

impl<H, T, P, R> RpcHandlerWrapperTrait for RpcHandlerWrapper<H, T, P, R>
where
	H: RpcHandler<T, P, R> + Clone + Send + Sync + 'static,
	T: Send + Sync + 'static,
	P: Send + Sync + 'static,
	R: Send + Sync + 'static,
{
	fn call(
		&self,
		rpc_resources: RpcResources,
		params: Option<Value>,
	) -> Pin<Box<dyn Future<Output = Result<Value>> + Send>> {
		Box::pin(self.call(rpc_resources, params))
	}
}
