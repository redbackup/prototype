use super::schema::*;
use chrono::prelude::*;

#[derive(Queryable,Insertable,Identifiable,PartialEq,Debug,Hash,Eq)]
#[table_name = "snapshots"]
#[primary_key(uuid)]
pub struct Snapshot {
    pub uuid: String, // TODO: Uuid?
    pub creation_date: NaiveDateTime,
}
