use r2d2::{Config,Pool};
use diesel::sqlite::SqliteConnection;
use r2d2_diesel::ConnectionManager;
use chrono::prelude::*;
use self::diesel::prelude::*;
use r2d2;
use diesel;
use uuid::Uuid;

use self::snapshot::Snapshot;
use self::schema::snapshots;

mod snapshot;
mod chunk;
mod snapshotchunk;
mod schema;
#[cfg(test)] mod tests;


embed_migrations!("migrations");

quick_error! {
    #[derive(Debug)]
    pub enum DatabaseError {
        PoolInitializationError(err: r2d2::InitializationError) {
            from()
        }
        ConnectionError(err: r2d2::GetTimeout) {
            from()
        }
        QueryError(err: diesel::result::Error) {
            from()
        }
        MigrationError(err: diesel::migrations::RunMigrationsError) {
            from()
        }
    }
}

pub struct ChunkIndex {
    db_pool: Pool<ConnectionManager<SqliteConnection>>
}

impl ChunkIndex {
    fn new(database_url: &str) -> Result<Self, DatabaseError> {
        let config = Config::default();
        let manager = ConnectionManager::<SqliteConnection>::new(database_url);
        let db_pool = Pool::new(config, manager)?;
        let conn = db_pool.get()?;

        embedded_migrations::run(&*conn)?;

        Ok(ChunkIndex { db_pool })
    }

    fn add_snapshot(&self, creation_date: NaiveDateTime, expiration_date: NaiveDateTime) -> Result<Snapshot,DatabaseError> {
        let conn = self.db_pool.get()?;
        let new_snapshot = Snapshot {
            uuid: Uuid::new_v4().hyphenated().to_string(),
            creation_date,
            expiration_date,
        };

        diesel::insert(&new_snapshot).into(snapshots::table).execute(&*conn)?;
        snapshots::dsl::snapshots.find(&new_snapshot.uuid).first::<Snapshot>(&*conn).map_err(|e| DatabaseError::from(e))
    }
}
