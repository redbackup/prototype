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
use self::schema::snapshots::dsl::*;

use self::chunk::Chunk;
use self::schema::chunks;
use self::schema::chunks::dsl::*;

mod snapshot;
mod chunk;
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

    fn add_snapshot(&self, _creation_date: NaiveDateTime) -> Result<Snapshot,DatabaseError> {
        let conn = self.db_pool.get()?;
        let new_snapshot = Snapshot {
            uuid: Uuid::new_v4().hyphenated().to_string(),
            creation_date: _creation_date,
        };

        diesel::insert(&new_snapshot).into(snapshots::table).execute(&*conn)?;
        snapshots.find(&new_snapshot.uuid).first::<Snapshot>(&*conn).map_err(|e| DatabaseError::from(e))
    }


    fn add_or_update_chunk(&self, chunk: &Chunk) -> Result<Chunk,DatabaseError> {
        let conn = self.db_pool.get()?;
        //TODO: Snapshot - chunk relation is not that easy...
        if let Ok(existing_chunk) = chunks.find((&chunk.file, &chunk.chunk_identifier)).first::<Chunk>(&*conn) {
            if existing_chunk.expiration_date < chunk.expiration_date {
                diesel::update(chunk).set(expiration_date.eq(chunk.expiration_date))
                    .execute(&*conn)?;
            }
        } else {
            diesel::insert(chunk).into(chunks::table).execute(&*conn)?;
        }

        chunks.find((&chunk.file, &chunk.chunk_identifier)).first::<Chunk>(&*conn).map_err(|e| DatabaseError::from(e))
    }

}
