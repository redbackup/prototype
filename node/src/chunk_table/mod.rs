use r2d2::{Config, Pool, PooledConnection};
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
            display("Pool initialization failed ({})", err)
            cause(err)
        }
        ConnectionError(err: r2d2::GetTimeout) {
            from()
            display("Connection error ({})", err)
            cause(err)
        }
        QueryError(err: diesel::result::Error) {
            from()
            display("Query Error ({})", err)
            cause(err)
        }
        MigrationError(err: diesel::migrations::RunMigrationsError) {
            from()
            display("Migration error ({})", err)
            cause(err)
        }
    }
}

/// Represents the chunk table, where the node persists information about chunks it holds.
pub struct ChunkTable {
    db_pool: Pool<ConnectionManager<SqliteConnection>>,
}

impl Clone for ChunkTable {
    fn clone(&self) -> Self {
        ChunkTable { db_pool: self.db_pool.clone() }
    }
}

impl ChunkTable {
    pub fn new(database_url: &str) -> Result<Self, DatabaseError> {
        debug!("Connect to chunk table database {}", database_url);
        {
            let config = Config::default();
            let manager = ConnectionManager::<SqliteConnection>::new(database_url);
            let first_db_pool = Pool::new(config, manager)?;
            let conn = first_db_pool.get()?;
            debug!("Run chunk table database migrations");
            embedded_migrations::run(&*conn)?;

        }

        let config = Config::default();
        let manager = ConnectionManager::<SqliteConnection>::new(database_url);
        let db_pool = Pool::new(config, manager)?;

        debug!("Finished creating chunk table");
        Ok(ChunkTable { db_pool })
    }

    pub fn get_db_connection(
        &self,
    ) -> Result<PooledConnection<ConnectionManager<SqliteConnection>>, DatabaseError> {
        let conn = self.db_pool.get()?;
        // Make sure the database is opened in Write-Ahead-Log mode.
        // Note that we currently have no way to detect if this failed.
        conn.execute("PRAGMA journal_mode=WAL;")?;
        conn.execute("PRAGMA synchronous=0;")?;
        conn.execute("PRAGMA wal_autocheckpoint==0;")?;
        conn.execute("PRAGMA wal_checkpoint(PASSIVE);")?;
        Ok(conn)
    }

    pub fn get_chunk(&self, chunk_identifier: &str) -> Result<Chunk, DatabaseError> {
        let conn = self.get_db_connection()?;
        chunks::dsl::chunks
            .find(chunk_identifier)
            .first(&*conn)
            .map_err(|e| DatabaseError::from(e))
    }

    pub fn get_root_handles(&self) -> Result<Vec<Chunk>, DatabaseError> {
        let conn = self.get_db_connection()?;
        chunks::dsl::chunks
            .filter(chunks::dsl::root_handle.eq(true))
            .load(&*conn)
            .map_err(|e| DatabaseError::from(e))
    }

    pub fn load_random_chunks(&self, number_of_chunks: i64) -> Result<Vec<Chunk>, DatabaseError> {
        let conn = self.get_db_connection()?;
        chunks::dsl::chunks
            .order(RANDOM)
            .limit(number_of_chunks)
            .load(&*conn)
            .map_err(|e| DatabaseError::from(e))
    }

    /// Update a chunk in the database (postpone the expiration date if appropriate).
    pub fn update_chunk(&self, chunky: &Chunk) -> Result<Chunk, DatabaseError> {
        let conn = self.get_db_connection()?;
        trace!("Update chunk {} as transaction", chunky.chunk_identifier);
        conn.transaction::<_, DatabaseError, _>(|| {
            let db_chunk: Chunk = chunks::dsl::chunks.find(&chunky.chunk_identifier).first(
                &*conn,
            )?;

            // Evaluate if there are changes that require a update query
            let mut changed = false;
            let mut expiration_date = db_chunk.expiration_date;

            if expiration_date < chunky.expiration_date {
                expiration_date = chunky.expiration_date;
                changed = true;
            }

            let root_handle = db_chunk.root_handle || chunky.root_handle;

            changed = changed || root_handle != db_chunk.root_handle;

            if changed {
                trace!(
                    "Write updated chunk {} to database",
                    chunky.chunk_identifier
                );
                diesel::update(chunks::dsl::chunks.find(&chunky.chunk_identifier))
                    .set((
                        chunks::dsl::expiration_date.eq(expiration_date),
                        chunks::dsl::root_handle.eq(root_handle),
                    ))
                    .execute(&*conn)?;

                // Get updated chunk, as SQLite does not support RETURNING clauses.
                chunks::dsl::chunks
                    .find(&chunky.chunk_identifier)
                    .first::<Chunk>(&*conn)
                    .map_err(|e| DatabaseError::from(e))
            } else {
                Ok(db_chunk)
            }
        })
    }

    /// Update multiple chunks and return them as stored in the database.
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
        let conn = self.get_db_connection()?;

        diesel::insert(new_chunk).into(chunks::table).execute(
            &*conn,
        )?;

        chunks::dsl::chunks
            .find(&new_chunk.chunk_identifier)
            .first::<Chunk>(&*conn)
            .map_err(|e| DatabaseError::from(e))
    }
}
