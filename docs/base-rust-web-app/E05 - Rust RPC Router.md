# Latest RpcRouter

## Resources

 - https://www.youtube.com/watch?v=Gc5Nj5LJe1U

## Introduction

The library [rpc-router](https://github.com/jeremychone/rust-rpc-router) on which his `rust10x` code is built is somewhat different in details. See _Original Video Summary_ below for the starting point of this library.

There's a lot of low-level stuff happening here, things that'd be great to learn from a library-writer's perspective (_Stuff that I was fairly comfortable with in the C++ world_).

## Starting point

I am taking a usage snippet from the `conv/agent` model in his demo.

```rust
pub fn rpc_router_builder() -> RouterBuilder {
	router_builder!(
		// Same as RpcRouter::new().add...
		create_conv,
		get_conv,
		list_convs,
		update_conv,
		delete_conv,
		add_conv_msg,
	)
}

generate_common_rpc_fns!(
	Bmc: ConvBmc,
	Entity: Conv,
	ForCreate: ConvForCreate,
	ForUpdate: ConvForUpdate,
	Filter: ConvFilter,
	Suffix: conv
);

/// Returns conv_msg
pub async fn add_conv_msg(
	ctx: Ctx,
	mm: ModelManager,
	params: ParamsForCreate<ConvMsgForCreate>,
) -> Result<DataRpcResult<ConvMsg>> {
	let ParamsForCreate { data: msg_c } = params;

	let msg_id = ConvBmc::add_msg(&ctx, &mm, msg_c).await?;
	let msg = ConvBmc::get_msg(&ctx, &mm, msg_id).await?;

	Ok(msg.into())
}
```

## Macro expansion of router_builder!

```rust
/// A simple macro to create a new RouterBuider from a list of handlers
/// and optionaly a list of resources
///
/// ## Pattern 1 - List of function handlers
/// ```
/// router_builder!(
///   create_project,
///   list_projects,
///   update_project,
///   delete_project
/// );
/// ```
/// Is equivalent to:
/// ```
/// RouterBuilder::default()
///     .append_dyn("create_project", create_project.into_box())
///     .append_dyn("list_projects", list_projects.into_box())
///     .append_dyn("update_project", update_project.into_box())
///     .append_dyn("delete_project", delete_project.into_box())
/// ```
///
/// ## Pattern 2 - List of function handlers, and resources
/// ```
/// router_builder!(
///   handlers: [get_task, create_task],         // will be turned into routes
///   resources: [ModelManager {}, AiManager {}] // common resources for all calls
/// );
/// ```
///
/// Is equivalent to:
///
/// ```
/// RouterBuilder::default()
///     .append_dyn("get_task", get_task.into_box())
///     .append_dyn("create_task", create_task.into_box())
///     .append_resource(ModelManager {})
///     .append_resource(AiManager {})
/// ```
///
/// ## Pattern 3 - Just for consistency with Pattern 2, we can have omit the resources
///
/// ```
/// router_builder!(
///   handlers: [get_task, create_task]
/// );
/// ```
///
#[macro_export]
macro_rules! router_builder {
	// Pattern 1 - with `rpc_router!(my_fn1, myfn2)`
    ($($fn_name:ident),+ $(,)?) => {
        {
					use rpc_router::{Handler, RouterBuilder};

					let mut builder = RouterBuilder::default();
					$(
							builder = builder.append_dyn(stringify!($fn_name), $fn_name.into_dyn());
					)+
					builder
        }
    };

    // Pattern 2 - `rpc_router!(handlers: [my_fn1, myfn2], resources: [ModelManger {}, AiManager {}])`
    (handlers: [$($handler:ident),* $(,)?], resources: [$($resource:expr),* $(,)?]) => {{
        use rpc_router::{Handler, RouterBuilder};

        let mut builder = RouterBuilder::default();
        $(
            builder = builder.append_dyn(stringify!($handler), $handler.into_dyn());
        )*
        $(
            builder = builder.append_resource($resource);
        )*
        builder
    }};

    // Pattern 3 - with `rpc_router!(handlers: [my_fn1, myfn2])`
    (handlers: [$($handler:ident),* $(,)?]) => {{
        use rpc_router::{Handler, RouterBuilder};

        let mut builder = RouterBuilder::default();
        $(
            builder = builder.append_dyn(stringify!($handler), $handler.into_dyn());
        )*
        builder
    }};
}
```

## Macro expansion of generate_common_rpc_fns!

```rust
/// Create the base crud rpc functions following the common pattern.
/// - `create_...`
/// - `get_...`
///
/// NOTE: Make sure to import the Ctx, ModelManager, ... in the model that uses this macro.
#[macro_export]
macro_rules! generate_common_rpc_fns {
    (
        Bmc: $bmc:ident,
        Entity: $entity:ty,
        ForCreate: $for_create:ty,
        ForUpdate: $for_update:ty,
        Filter: $filter:ty,
        Suffix: $suffix:ident
    ) => {
        paste! {
            pub async fn [<create_ $suffix>](
                ctx: Ctx,
                mm: ModelManager,
                params: ParamsForCreate<$for_create>,
            ) -> Result<DataRpcResult<$entity>> {
                let ParamsForCreate { data } = params;
                let id = $bmc::create(&ctx, &mm, data).await?;
                let entity = $bmc::get(&ctx, &mm, id).await?;
                Ok(entity.into())
            }

            pub async fn [<get_ $suffix>](
                ctx: Ctx,
                mm: ModelManager,
                params: ParamsIded,
            ) -> Result<DataRpcResult<$entity>> {
                let entity = $bmc::get(&ctx, &mm, params.id).await?;
                Ok(entity.into())
            }

            // Note: for now just add `s` after the suffix.
            pub async fn [<list_ $suffix s>](
                ctx: Ctx,
                mm: ModelManager,
                params: ParamsList<$filter>,
            ) -> Result<DataRpcResult<Vec<$entity>>> {
                let entities = $bmc::list(&ctx, &mm, params.filters, params.list_options).await?;
                Ok(entities.into())
            }

            pub async fn [<update_ $suffix>](
                ctx: Ctx,
                mm: ModelManager,
                params: ParamsForUpdate<$for_update>,
            ) -> Result<DataRpcResult<$entity>> {
                let ParamsForUpdate { id, data } = params;
                $bmc::update(&ctx, &mm, id, data).await?;
                let entity = $bmc::get(&ctx, &mm, id).await?;
                Ok(entity.into())
            }

            pub async fn [<delete_ $suffix>](
                ctx: Ctx,
                mm: ModelManager,
                params: ParamsIded,
            ) -> Result<DataRpcResult<$entity>> {
                let ParamsIded { id } = params;
                let entity = $bmc::get(&ctx, &mm, id).await?;
                $bmc::delete(&ctx, &mm, id).await?;
                Ok(entity.into())
            }
        }
    };
}
```

## RouterBuilder

```rust
impl RouterBuilder {
	/// Add a dyn_handler to the router builder.
	///
	/// ```
	/// RouterBuilder::default().add_dyn("method_name", my_handler_fn.into_dyn());
	/// ```
	///
	/// Note: This is the preferred way to add handlers to the router, as it
	///       avoids monomorphization of the add function.
	///       The `RouterInner` also has a `.add()` as a convenience function to just pass the function.
	///       See `RouterInner::add` for more details.
	pub fn append_dyn(mut self, name: &'static str, dyn_handler: Box<dyn RpcHandlerWrapperTrait>) -> Self {
		self.inner.append_dyn(name, dyn_handler);
		self
	}

	/// Add a route (name, handler function) to the builder
	///
	/// ```
	/// RouterBuilder::default().add("method_name", my_handler_fn);
	/// ```
	///
	/// Note: This is a convenient add function variant with generics,
	///       and there will be monomorphed versions of this function
	///       for each type passed. Use `RouterInner::add_dyn` to avoid this.
	pub fn append<F, T, P, R>(mut self, name: &'static str, handler: F) -> Self
	where
		F: Handler<T, P, R> + Clone + Send + Sync + 'static,
		T: Send + Sync + 'static,
		P: Send + Sync + 'static,
		R: Send + Sync + 'static,
	{
		self.inner.append_dyn(name, handler.into_dyn());
		self
	}
```

## RpcHandlerWrapper trait

```rust
use crate::handler::PinFutureValue;
use crate::Handler;
use crate::{Resources, Result};
use futures::Future;
use serde_json::Value;
use std::marker::PhantomData;
use std::pin::Pin;

/// `RpcHandlerWrapper` is an `RpcHandler` wrapper that implements
/// `RpcHandlerWrapperTrait` for type erasure, enabling dynamic dispatch.
/// Generics:
/// - `H`: The handler trait for the function
/// - `K`: The Resources, meaning the type passed in the call that has the `FromResources` trait for the various `T` types (cannot use `R`, as it is reserved for the
/// - `T`: The type (can be a tuple when multiple) for the function parameters
/// - `P`: The JSON RPC parameter
/// - `R`: The response type
///
/// Thus, all these types except `H` will match the generic of the `H` handler trait. We keep them in phantom data.
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
	H: Handler<T, P, R> + Send + Sync + 'static,
	T: Send + Sync + 'static,
	P: Send + Sync + 'static,
	R: Send + Sync + 'static,
{
	pub fn call(&self, rpc_resources: Resources, params: Option<Value>) -> H::Future {
		// Note: Since handler is a FnOnce, we can use it only once, so we clone it.
		//       This is likely optimized by the compiler.
		let handler = self.handler.clone();
		Handler::call(handler, rpc_resources, params)
	}
}

/// `RpcHandlerWrapperTrait` enables `RpcHandlerWrapper` to become a trait object,
/// allowing for dynamic dispatch.
pub trait RpcHandlerWrapperTrait: Send + Sync {
	fn call(&self, rpc_resources: Resources, params: Option<Value>) -> PinFutureValue;
}

impl<H, T, P, R> RpcHandlerWrapperTrait for RpcHandlerWrapper<H, T, P, R>
where
	H: Handler<T, P, R> + Clone + Send + Sync + 'static,
	T: Send + Sync + 'static,
	P: Send + Sync + 'static,
	R: Send + Sync + 'static,
{
	fn call(
		&self,
		rpc_resources: Resources,
		params: Option<Value>,
	) -> Pin<Box<dyn Future<Output = Result<Value>> + Send>> {
		Box::pin(self.call(rpc_resources, params))
	}
}
```

## Handler trait

```rust
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
pub trait Handler<T, P, R>: Clone
where
	T: Send + Sync + 'static,
	P: Send + Sync + 'static,
	R: Send + Sync + 'static,
{
	/// The type of future calling this handler returns.
	type Future: Future<Output = Result<Value>> + Send + 'static;

	/// Call the handler.
	fn call(self, rpc_resources: Resources, params: Option<Value>) -> Self::Future;

	/// Convert this RpcHandler into a Boxed dyn RpcHandlerWrapperTrait,
	/// for dynamic dispatch by the Router.
	fn into_dyn(self) -> Box<dyn RpcHandlerWrapperTrait>
	where
		Self: Sized + Send + Sync + 'static,
	
```

## IntoParams trait

```rust
use crate::handler::PinFutureValue;
use crate::Handler;
use crate::{Resources, Result};
use futures::Future;
use serde_json::Value;
use std::marker::PhantomData;
use std::pin::Pin;

/// `RpcHandlerWrapper` is an `RpcHandler` wrapper that implements
/// `RpcHandlerWrapperTrait` for type erasure, enabling dynamic dispatch.
/// Generics:
/// - `H`: The handler trait for the function
/// - `K`: The Resources, meaning the type passed in the call that has the `FromResources` trait for the various `T` types (cannot use `R`, as it is reserved for the
/// - `T`: The type (can be a tuple when multiple) for the function parameters
/// - `P`: The JSON RPC parameter
/// - `R`: The response type
///
/// Thus, all these types except `H` will match the generic of the `H` handler trait. We keep them in phantom data.
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
	H: Handler<T, P, R> + Send + Sync + 'static,
	T: Send + Sync + 'static,
	P: Send + Sync + 'static,
	R: Send + Sync + 'static,
{
	pub fn call(&self, rpc_resources: Resources, params: Option<Value>) -> H::Future {
		// Note: Since handler is a FnOnce, we can use it only once, so we clone it.
		//       This is likely optimized by the compiler.
		let handler = self.handler.clone();
		Handler::call(handler, rpc_resources, params)
	}
}

/// `RpcHandlerWrapperTrait` enables `RpcHandlerWrapper` to become a trait object,
/// allowing for dynamic dispatch.
pub trait RpcHandlerWrapperTrait: Send + Sync {
	fn call(&self, rpc_resources: Resources, params: Option<Value>) -> PinFutureValue;
}

impl<H, T, P, R> RpcHandlerWrapperTrait for RpcHandlerWrapper<H, T, P, R>
where
	H: Handler<T, P, R> + Clone + Send + Sync + 'static,
	T: Send + Sync + 'static,
	P: Send + Sync + 'static,
	R: Send + Sync + 'static,
{
	fn call(
		&self,
		rpc_resources: Resources,
		params: Option<Value>,
	) -> Pin<Box<dyn Future<Output = Result<Value>> + Send>> {
		Box::pin(self.call(rpc_resources, params))
	}
}
```

## Actual call mechanism

Once the JSON RPC is parsed, imagine the follwing

```rust
let req_create_conv = hc.do_post(
		"/api/rpc",
		json!({
			"jsonrpc": "2.0",
			"id": 1,
			"method": "add_conv_msg",
			"params": {
				"data": {
					"conv_id": conv_id,
					"content": "This is the first comment"
				}
			}
		}),
	);
```

The rpc method is called via the snippet below (_from rpc-roouter_). This means, the `params` key is extracted to send to `call_route`

```rust
pub async fn call(&self, resources: Resources, rpc_request: Request) -> CallResult {
		let Request { id, method, params } = rpc_request;

		self.call_route(resources, Some(id), method, params).await
	}
```

`call_route` looks like this

```rust
/// Performs the RPC call for a given Request object, which contains the `id`, method name, and parameters.
///
/// - method: The json-rpc method name.
/// -     id: The json-rpc request `.id`, which should be sent by the client.
///           It is required to echo it back in the json-rpc response.
///           Can be `Value::Null`, and if None, it will be set to `Value::Null`
/// - params: The optional json-rpc params
///
/// Returns an ResponseResult, where either the success value (Response) or the error (ResponseError)
/// will echo back the `id` and `method` part of their construct
pub async fn call_route(
	&self,
	resources: Resources,
	id: Option<Value>,
	method: impl Into<String>,
	params: Option<Value>,
) -> CallResult {
	let method = method.into();
	let id = id.unwrap_or(Value::Null);

	if let Some(route) = self.route_by_name.get(method.as_str()) {
		match route.call(resources, params).await {
			Ok(value) => Ok(CallResponse { id, method, value }),
			Err(error) => Err(CallError { id, method, error }),
		}
	} else {
		Err(CallError {
			id,
			method,
			error: Error::MethodUnknown,
		})
	}
}
```

# Original Video Summary

> Note: This portion has already been split into his own rpc-router crate at https://github.com/jeremychone/rust-rpc-router. I will paste snippets from that alongside the video notes.

 - RpcRouter
 - TimeStamp _add to all entities_  (next video)
 - ReqStamp _to have a duration for the request_ (next video)


# Learning Process

I am trying to watch this out of order to see if I can make sense of it. My main goal is to get started treating this as a programattic gRPC and flesh the API out to serve an AI model via candle. Don't need the rpc params to be serializable to a DB.

Just a 10min video so nothing lost.

# Axum inspiration

```rust
pub fn rpc_router() -> RpcRouter {
    rpc_router!(
        create_task,
        list_task,
        update_task,
        delete_task
    )
}

pub async fn create_task(
    ctx : Ctx,
    mm : ModelManager,
    params : ParamsForCreate<ProjectForCreate>,
) -> Result<Task> {
}
```

 - `rpc_router` is the macro impl.
 - `rpc_router` invocation is equivalent to `RpcRouter::new().add(funcName)`
 - Just like axum handlers, this macro, extracts the args and pushes them in any order. i.e., `create_tas(ctx, mm)` or `create_task(mm, ctx)` work equally well. _Confusing to newbies but whatever_. The macro however, enforces that the last argument is the params (_how does it figure that ?_)



# RpcRouter

## Function Design goals

Each rpc function has two sections of the arguments

 - A _variable portion with resources_: context and other setup params that can be optional and specified in any order
   - These are server/request life span items so no serialization or db storage needed.
 - A _fixed portion_ with 0 or 1 params for the RPC function.
   - Enforce these to be the last parameter is present.
   - As a general facility, good to have serialization and de-serialization and possible DB storage for these.

## Macro expansion

The macro expansion of 

```rust
pub fn rpc_router() -> RpcRouter {
    rpc_router!(
        create_task,
        list_task,
        update_task,
        delete_task
    )
}
```

is

```rust
pub fn rpc_router() -> RpcRouter {
    RpcRouter::new()
        .add(create_task)
        .add(list_taks)
        .add(update_taks)
        .add(delete_task)
}
```

## Resource portion of the RPC call parameters

This is an illustration. The actual macro impl is more complex but similar. _The final code on github is the rust versio of what I did at work for `AttributeManager.getAttribute(&myAttr)` which was storing the attributes keyed by their RTTI typeId and later a multi-map_.

```rust
use crate::router::FromResources;

pub struct RpcResources {
    pub mm: ModelManager,
    pub ctx: Option<Ctx>,
}

impl FromResources for Ctx {

}

impl FromResources for Option<Ctx> {

}

impl FromResources for ModelManager {

}
```

 - Likely, figure out members of the RpcResources based on signatures of the methods used
 - One/two methods for each based on whether they are being expected as values or as `Option<type>`
 - `impl FromResources for ModelManager` adds the methods of `FromResources` trait to the `ModelManager struct`. 
 	- ❓However, rust is all compiled. If `Option` is a known type in some other crate, how can you extend it in another crate as a static compiled language ?
	- ✔️ You have to include these at the call-site I guess.

## Payload portion of the RPC call

 This is typically the parameters that come from a client call.

 ```rust
 /// `IntoParams` allows for converting an `Option<Value>` into
/// the necessary type for RPC handler parameters.
/// The default implementation below will result in failure if the value is `None`.
/// For customized behavior, users can implement their own `into_params`
/// method.
pub trait IntoParams: DeserializeOwned + Send {
	fn into_params(value: Option<Value>) -> Result<Self> {
		match value {
			Some(value) => Ok(serde_json::from_value(value).map_err(Error::ParamsParsing)?),
			None => Err(Error::ParamsMissingButRequested),
		}
	}
}

#[derive(Deserialize)]
 pub struct ParamsForCreate<D> {
    data: D,
 }

 impl<D> IntoParams for ParamsForCreate<D> where D: DeserializeOwned + Send {    
 }
 ```

 - Usually no need to provide an impl if we already expect `D` to be `DeserializeOwned`
 - `Value` in `into_params(value: Option<Value>)` seems to be a JSon serialized value.



# Full flow

Examne the web side routes

```rust
// src/main.rs ------------------------------------------------------
async fn main() -> {
    let routes_rpc = web::routes_rpc::routes(mm.clone())
    .route_layer(middleware::from_fn(mw_ctx_require));
}

// src/web/routes_rpc.rs -------------------------------------------
pub fn routes(mm: ModelManager) -> Router {
	// Build the combined Rpc Router (from `rpc-router` crate)
	let rpc_router = all_rpc_router_builder()
		// Add the common resources for all rpc calls
		.append_resource(mm)
		.build();

	// Build the Axum Router for '/rpc'
	Router::new()
		.route("/rpc", post(rpc_axum_handler))
		.with_state(rpc_router)
}

async fn rpc_axum_handler(
	State(rpc_router): State<rpc_router::Router>,
	ctx: CtxW,
	Json(rpc_req): Json<Value>,
) -> Response {
	let ctx = ctx.0;

	// -- Parse and RpcRequest validate the rpc_request
	let rpc_req = match rpc_router::Request::try_from(rpc_req) {
		Ok(rpc_req) => rpc_req,
		Err(rpc_req_error) => {
			let res =
				crate::web::Error::RpcRequestParsing(rpc_req_error).into_response();
			return res;
		}
	};

	// -- Create the RPC Info
	//    (will be set to the response.extensions)
	let rpc_info = RpcInfo {
		id: Some(rpc_req.id.clone()),
		method: rpc_req.method.clone(),
	};

	// -- Add the request specific resources
	// Note: Since Ctx is per axum request, we construct additional RPC resources.
	//       These additional resources will be "overlayed" on top of the base router services,
	//       meaning they will take precedence over the base router ones, but won't replace them.
	let additional_resources = resources_builder![ctx].build();

	// -- Exec Rpc Route
	let rpc_call_result = rpc_router
		.call_with_resources(rpc_req, additional_resources)
		.await;

	// -- Build Json Rpc Success Response
	// Note: Error Json response will be generated in the mw_res_map as wil other error.
	let res = rpc_call_result.map(|rpc_call_response| {
		let body_response = json!({
			"jsonrpc": "2.0",
			"id": rpc_call_response.id,
			"result": rpc_call_response.value
		});
		Json(body_response)
	});

	// -- Create and Update Axum Response
	// Note: We store data in the Axum Response extensions so that
	//       we can unpack it in the `mw_res_map` for client-side rendering.
	//       This approach centralizes error handling for the client at the `mw_res_map` module
	let res: crate::web::Result<_> = res.map_err(crate::web::Error::from);
	let mut res = res.into_response();
	// Note: Here, add the capture RpcInfo (RPC ID and method) into the Axum response to be used
	//       later in the `mw_res_map` for RequestLineLogging, and eventual JSON-RPC error serialization.
	res.extensions_mut().insert(Arc::new(rpc_info));

	res
}

```

`/api/rpc/` -> `rpc_axum_handler` ->`rpc_router.call_with_resources()`


