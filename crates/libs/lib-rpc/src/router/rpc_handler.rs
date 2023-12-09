use crate::router::into_params::IntoParams;
use crate::router::FromResources;
use crate::router::PinFutureValue;
use crate::router::Result;
use crate::router::RpcHandlerWrapper;
use crate::router::RpcHandlerWrapperTrait;
use crate::RpcResources;
use futures::Future;
use serde::Serialize;
use serde_json::Value;

/// The `Handler` trait that will be implemented by rpc handler functions.
///
/// Key points:
/// - Rpc handler functions are asynchronous, thus returning a Future of Result<Value>.
/// - The call format is normalized to two `impl FromResources` arguments (for now) and one optionals  `impl IntoParams`, which represent the json-rpc's optional value.
/// - `into_box` is a convenient method for converting a RpcHandler into a Boxed dyn RpcHandlerWrapperTrait,
///   allowing for dynamic dispatch by the Router.
/// - A `RpcHandler` will typically be implemented for static functions, as `FnOnce`,
///   enabling them to be cloned with none or negligible performance impact,
///   thus facilitating the use of RpcRoute dynamic dispatch.
/// - `T` is the tuple of `impl FromResources` arguments.
/// - `P` is the `impl IntoParams` argument.
///
pub trait RpcHandler<T, P, R>: Clone
where
	T: Send + Sync + 'static,
	P: Send + Sync + 'static,
	R: Send + Sync + 'static,
{
	/// The type of future calling this handler returns.
	type Future: Future<Output = Result<Value>> + Send + 'static;

	/// Call the handler.
	fn call(
		self,
		rpc_resources: RpcResources,
		params: Option<Value>,
	) -> Self::Future;

	/// Convert this RpcHandler into a Boxed dyn RpcHandlerWrapperTrait,
	/// for dynamic dispatch by the Router.
	fn into_dyn(self) -> Box<dyn RpcHandlerWrapperTrait>
	where
		Self: Sized + Send + Sync + 'static,
	{
		Box::new(RpcHandlerWrapper::new(self)) as Box<dyn RpcHandlerWrapperTrait>
	}
}

/// Macro generatring the RpcHandler implementations for zero or more FromResources with the last argument being IntoParams
/// and one with not last IntoParams argument.
macro_rules! impl_rpc_handler_pair {
    ($($T:ident),*) => {

				// RpcHandler implementations for zero or more FromResources with the last argument being IntoParams
        impl<F, Fut, $($T,)* P, R> RpcHandler<($($T,)*), (P,), R> for F
        where
            F: FnOnce($($T,)* P) -> Fut + Clone + Send + 'static,
            $( $T: FromResources + Send + Sync + 'static, )*
            P: IntoParams + Send + Sync + 'static,
            R: Serialize + Send + Sync + 'static,
            Fut: Future<Output = Result<R>> + Send,
        {
            type Future = PinFutureValue;

						#[allow(unused)] // somehow rpc_resources will be marked as unused
            fn call(
                self,
                rpc_resources: RpcResources,
                params_value: Option<Value>,
            ) -> Self::Future {
                Box::pin(async move {
                    let param = P::into_params(params_value)?;

                    let result = self(
                        $( $T::from_resources(&rpc_resources)?, )*
                        param,
                    )
                    .await?;
                    Ok(serde_json::to_value(result)?)
                })
            }
        }

				// RpcHandler implementations for zero or more FromResources and NO IntoParams
				impl<F, Fut, $($T,)* R> RpcHandler<($($T,)*), (), R> for F
				where
						F: FnOnce($($T,)*) -> Fut + Clone + Send + 'static,
						$( $T: FromResources + Send + Sync + 'static, )*
						R: Serialize + Send + Sync + 'static,
						Fut: Future<Output = Result<R>> + Send,
				{
						type Future = PinFutureValue;

						#[allow(unused)] // somehow rpc_resources will be marked as unused
						fn call(
								self,
								rpc_resources: RpcResources,
								_params: Option<Value>,
						) -> Self::Future {
								Box::pin(async move {
										let result = self(
												$( $T::from_resources(&rpc_resources)?, )*
										)
										.await?;
										Ok(serde_json::to_value(result)?)
								})
						}
				}
    };

}

impl_rpc_handler_pair!();
impl_rpc_handler_pair!(T1);
impl_rpc_handler_pair!(T1, T2);
impl_rpc_handler_pair!(T1, T2, T3);
impl_rpc_handler_pair!(T1, T2, T3, T4);
impl_rpc_handler_pair!(T1, T2, T3, T4, T5);
