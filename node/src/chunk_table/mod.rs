use r2d2::{Config,Pool};
use diesel::sqlite::SqliteConnection;
use r2d2_diesel::ConnectionManager;
use chrono::prelude::*;
use self::diesel::prelude::*;
use r2d2;
use diesel;

mod chunk;
mod schema;
#[cfg(test)] mod tests;

use self::chunk::Chunk;
use self::schema::chunks;

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

pub struct ChunkTable {
    db_pool: Pool<ConnectionManager<SqliteConnection>>
}

impl ChunkTable {
    pub fn new(database_url: &str) -> Result<Self, DatabaseError>{
        let config = Config::default();
        let manager = ConnectionManager::<SqliteConnection>::new(database_url);
        let db_pool = Pool::new(config, manager)?;
        let conn = db_pool.get()?;

        embedded_migrations::run(&*conn)?;

        Ok(ChunkTable { db_pool })
    }

    pub fn get_chunk(&self, chunk_identifier: &str) -> Result<Chunk, DatabaseError>{
        let conn = self.db_pool.get()?;
        chunks::dsl::chunks.find(chunk_identifier).first(&*conn).map_err(|e| DatabaseError::from(e))
    }

    pub fn remove_chunk(&self, chunk_identifier: &str) -> Result<usize, DatabaseError>{
        let conn = self.db_pool.get()?;
        diesel::delete(chunks::dsl::chunks.find(chunk_identifier)).execute(&*conn).map_err(|e| DatabaseError::from(e))
    }
    
    pub fn update_chunk(&self, chunk_identifier: &str,
                    expiration_date: NaiveDateTime, root_handle: bool) -> Result<Chunk, DatabaseError>{
        let conn = self.db_pool.get()?;
        conn.transaction::<_, DatabaseError, _>( || {
            let chunk: Chunk = chunks::dsl::chunks.find(chunk_identifier).first(&*conn)?;

            let mut expiration_date = expiration_date;
            if chunk.expiration_date > expiration_date {
                expiration_date = chunk.expiration_date;
            }
            let root_handle = chunk.root_handle || root_handle;

            diesel::update(chunks::dsl::chunks.find(chunk_identifier)).set((
                    chunks::dsl::expiration_date.eq(expiration_date),
                    chunks::dsl::root_handle.eq(root_handle)
                )).execute(&*conn)?;
                chunks::dsl::chunks.find(chunk_identifier).first::<Chunk>(&*conn).map_err(|e| DatabaseError::from(e))
            })
    }

    pub fn add_chunk(&self, chunk_identifier: &str,
                 expiration_date: NaiveDateTime, root_handle: bool) -> Result<Chunk, DatabaseError>{
        let conn = self.db_pool.get()?;
        let new_chunk = Chunk {
            chunk_identifier: String::from(chunk_identifier),
            expiration_date: expiration_date,
            root_handle: root_handle,
        };

        diesel::insert(&new_chunk).into(chunks::table).execute(&*conn)?;
        chunks::dsl::chunks.find(chunk_identifier).first::<Chunk>(&*conn).map_err(|e| DatabaseError::from(e))
    }
}

