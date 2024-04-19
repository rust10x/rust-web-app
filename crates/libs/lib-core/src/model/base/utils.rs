use crate::model::base::{CommonIden, DbBmc, TimestampIden};
use lib_utils::time::now_utc;
use modql::field::{SeaField, SeaFields};
use sea_query::IntoIden;

/// This method must be called when a model controller intends to create its entity.
pub fn prep_fields_for_create<MC>(fields: &mut SeaFields, user_id: i64)
where
	MC: DbBmc,
{
	if MC::has_owner_id() {
		fields.push(SeaField::new(CommonIden::OwnerId.into_iden(), user_id));
	}
	if MC::has_timestamps() {
		add_timestamps_for_create(fields, user_id);
	}
}

/// This method must be calledwhen a Model Controller plans to update its entity.
pub fn prep_fields_for_update<MC>(fields: &mut SeaFields, user_id: i64)
where
	MC: DbBmc,
{
	if MC::has_timestamps() {
		add_timestamps_for_update(fields, user_id);
	}
}

/// Update the timestamps info for create
/// (e.g., cid, ctime, and mid, mtime will be updated with the same values)
fn add_timestamps_for_create(fields: &mut SeaFields, user_id: i64) {
	let now = now_utc();
	fields.push(SeaField::new(TimestampIden::Cid, user_id));
	fields.push(SeaField::new(TimestampIden::Ctime, now));

	fields.push(SeaField::new(TimestampIden::Mid, user_id));
	fields.push(SeaField::new(TimestampIden::Mtime, now));
}

/// Update the timestamps info only for update.
/// (.e.g., only mid, mtime will be udpated)
fn add_timestamps_for_update(fields: &mut SeaFields, user_id: i64) {
	let now = now_utc();
	fields.push(SeaField::new(TimestampIden::Mid, user_id));
	fields.push(SeaField::new(TimestampIden::Mtime, now));
}
