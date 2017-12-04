use r2d2::{Config, Pool};
use diesel::sqlite::SqliteConnection;
use r2d2_diesel::ConnectionManager;
use self::diesel::prelude::*;
use r2d2;
use diesel;

mod chunk;
mod schema;

pub use self::chunk::Chunk;
use self::schema::chunks;

embed_migrations!("migrations");
no_arg_sql_function!(RANDOM, (), "Represents the sql RANDOM() function");

quick_error! {
    #[derive(Debug)]
    pub enum DatabaseError {
        PoolInitializationError(err: r2d2::InitializationError) {
            from()
            cause(err)
        }
        ConnectionError(err: r2d2::GetTimeout) {
            from()
            cause(err)
        }
        QueryError(err: diesel::result::Error) {
            from()
            cause(err)
        }
        MigrationError(err: diesel::migrations::RunMigrationsError) {
            from()
            cause(err)
        }
    }
}

pub struct ChunkTable {
    db_pool: Pool<ConnectionManager<SqliteConnection>>,
}

impl Clone for ChunkTable {
    fn clone(&self) -> Self {
        ChunkTable {
            db_pool: self.db_pool.clone(),
        }
    }
}

impl ChunkTable {
    pub fn new(database_url: &str) -> Result<Self, DatabaseError> {
        debug!("Connect to chunk table database {}", database_url);
        let config = Config::default();
        let manager = ConnectionManager::<SqliteConnection>::new(database_url);
        let db_pool = Pool::new(config, manager)?;
        let conn = db_pool.get()?;
        debug!("Run chunk table database migrations");
        embedded_migrations::run(&*conn)?;

        debug!("Finished creating chunk table");
        Ok(ChunkTable { db_pool })
    }

    #[allow(dead_code)]
    pub fn get_chunk(&self, chunk_identifier: &str) -> Result<Chunk, DatabaseError> {
        let conn = self.db_pool.get()?;
        chunks::dsl::chunks
            .find(chunk_identifier)
            .first(&*conn)
            .map_err(|e| DatabaseError::from(e))
    }

    pub fn get_root_handles(&self) -> Result<Vec<Chunk>, DatabaseError> {
        let conn = self.db_pool.get()?;
        chunks::dsl::chunks
            .filter(chunks::dsl::root_handle.eq(true))
            .load(&*conn)
            .map_err(|e| DatabaseError::from(e))
    }

    #[allow(dead_code)]
    pub fn remove_chunk(&self, chunk_identifier: &str) -> Result<usize, DatabaseError> {
        let conn = self.db_pool.get()?;
        diesel::delete(chunks::dsl::chunks.find(chunk_identifier))
            .execute(&*conn)
            .map_err(|e| DatabaseError::from(e))
    }

    pub fn load_random_chunks(&self, number_of_chunks: i64) -> Result<Vec<Chunk>, DatabaseError> {
        let conn = self.db_pool.get()?;
        chunks::dsl::chunks
            .order(RANDOM)
            .limit(number_of_chunks)
            .load(&*conn)
            .map_err(|e| DatabaseError::from(e))
    }

    pub fn update_chunk(&self, chunky: &Chunk) -> Result<Chunk, DatabaseError> {
        let conn = self.db_pool.get()?;
        trace!("Update chunk {} as transaction", chunky.chunk_identifier);
        conn.transaction::<_, DatabaseError, _>(|| {
            let db_chunk: Chunk = chunks::dsl::chunks
                .find(&chunky.chunk_identifier)
                .first(&*conn)?;

            let mut changed = false;
            let mut expiration_date = db_chunk.expiration_date;

            if expiration_date < chunky.expiration_date {
                expiration_date = chunky.expiration_date;
                changed = true;
            }

            let root_handle = db_chunk.root_handle || chunky.root_handle;

            changed = changed || root_handle != db_chunk.root_handle;

            if changed {
                trace!("Write updated chunk {} to database", chunky.chunk_identifier);
                diesel::update(chunks::dsl::chunks.find(&chunky.chunk_identifier))
                    .set((
                        chunks::dsl::expiration_date.eq(expiration_date),
                        chunks::dsl::root_handle.eq(root_handle),
                    ))
                    .execute(&*conn)?;

                chunks::dsl::chunks
                    .find(&chunky.chunk_identifier)
                    .first::<Chunk>(&*conn)
                    .map_err(|e| DatabaseError::from(e))
            } else {
                Ok(db_chunk)
            }
        })
    }
    pub fn get_and_update_chunks(&self, chunks: Vec<Chunk>) -> Result<Vec<Chunk>, DatabaseError> {
        let mut results: Vec<Chunk> = Vec::new();
        for chunk in chunks {
            if let Ok(chunk) = self.update_chunk(&chunk) {
                results.push(chunk);
            }
        }
        Ok(results)
    }

    pub fn add_chunk(&self, new_chunk: &Chunk) -> Result<Chunk, DatabaseError> {
        let conn = self.db_pool.get()?;

        diesel::insert(new_chunk)
            .into(chunks::table)
            .execute(&*conn)?;

        chunks::dsl::chunks
            .find(&new_chunk.chunk_identifier)
            .first::<Chunk>(&*conn)
            .map_err(|e| DatabaseError::from(e))
    }
}
