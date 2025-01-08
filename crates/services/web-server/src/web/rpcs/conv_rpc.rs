use lib_core::model::conv::{
	Conv, ConvBmc, ConvFilter, ConvForCreate, ConvForUpdate,
};
use lib_core::model::conv_msg::{ConvMsg, ConvMsgForCreate};
use lib_rpc_core::prelude::*;

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

/// Returns conv_msg
#[allow(unused)]
pub async fn get_conv_msg(
	ctx: Ctx,
	mm: ModelManager,
	params: ParamsIded,
) -> Result<DataRpcResult<ConvMsg>> {
	let ParamsIded { id: msg_id } = params;

	let msg = ConvBmc::get_msg(&ctx, &mm, msg_id).await?;

	Ok(msg.into())
}
