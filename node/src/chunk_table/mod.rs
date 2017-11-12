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

use self::chunk::{Chunk,NewChunk};
use self::schema::chunks;
use self::schema::chunks::dsl::*;

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
    fn new(database_url: &str) -> Result<Self, DatabaseError>{
        let config = Config::default();
        let manager = ConnectionManager::<SqliteConnection>::new(database_url);
        let db_pool = Pool::new(config, manager)?;
        let conn = db_pool.get()?;

        embedded_migrations::run(&*conn)?;

        Ok(ChunkTable { db_pool })
    }

    fn get_chunk(&self, _chunk_identifier: &str) -> Result<Chunk, DatabaseError>{
        let conn = self.db_pool.get()?;
        chunks.find(_chunk_identifier).first(&*conn).map_err(|e| DatabaseError::from(e))
    }

    fn remove_chunk(&self, _chunk_identifier: &str) -> Result<usize, DatabaseError>{
        let conn = self.db_pool.get()?;
        diesel::delete(chunks.find(_chunk_identifier)).execute(&*conn).map_err(|e| DatabaseError::from(e))
    }
    
    fn update_chunk(&self, _chunk_identifier: &str,
                    _expiration_date: NaiveDateTime, _root_handle: bool) -> Result<Chunk, DatabaseError>{
        let conn = self.db_pool.get()?;
        conn.transaction::<_, DatabaseError, _>( || {
            let chunk: Chunk = chunks.find(_chunk_identifier).first(&*conn)?;

            let mut _expiration_date = _expiration_date;
            if chunk.expiration_date > _expiration_date {
                _expiration_date = chunk.expiration_date;
            }
            let _root_handle = chunk.root_handle || _root_handle;

            diesel::update(chunks.find(_chunk_identifier)).set((
                    expiration_date.eq(_expiration_date),
                    root_handle.eq(_root_handle)
                )).execute(&*conn)?;
                chunks.find(_chunk_identifier).first::<Chunk>(&*conn).map_err(|e| DatabaseError::from(e))
            })
    }

    fn add_chunk(&self, _chunk_identifier: &str,
                 _expiration_date: NaiveDateTime, _root_handle: bool) -> Result<Chunk, DatabaseError>{
        let conn = self.db_pool.get()?;
        let new_chunk = NewChunk {
            chunk_identifier: _chunk_identifier,
            expiration_date: _expiration_date,
            root_handle: _root_handle,
        };

        diesel::insert(&new_chunk).into(chunks::table).execute(&*conn)?;
        chunks.find(_chunk_identifier).first::<Chunk>(&*conn).map_err(|e| DatabaseError::from(e))
    }

}

