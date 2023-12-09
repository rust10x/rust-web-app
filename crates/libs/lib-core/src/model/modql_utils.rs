use time::serde::rfc3339;

pub fn time_to_sea_value(
	json_value: serde_json::Value,
) -> modql::filter::SeaResult<sea_query::Value> {
	Ok(rfc3339::deserialize(json_value)?.into())
}
