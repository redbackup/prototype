use r2d2::{Config,Pool};
use diesel::sqlite::SqliteConnection;
use r2d2_diesel::ConnectionManager;
use chrono::prelude::*;
use self::diesel::prelude::*;
use r2d2;
use diesel;

use super::chunk::{Chunk,NewChunk};
use super::schema::chunks;
use super::schema::chunks::dsl::*;

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
        let db_pool = Pool::new(config, manager)?;//.map_err(|e| PoolInitializationError(e))?;
        let conn = db_pool.get()?;//.map_err(|e| ConnectionError(e))?;

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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[allow(unused_must_use)] // as we are not interested in the result of fs::remove_file
    fn _prepare_chunk_table(test_name: &str) -> ChunkTable {
        let database_url = format!("{}/test-database-node-{}.db", env!("OUT_DIR"), test_name);
        fs::remove_file(&database_url);
        println!("Database file: {}", &database_url);
        ChunkTable::new(&database_url).expect("Chunk table could not be created")
    }

    fn _prepare_one_chunk(chunk_table: &ChunkTable) -> Chunk {
        // Note that tests might depend on these concrete values!
        let identifier = String::from("7fcaddc8772aaa616f43361c217c23d308e933465b2099d00ba1418fec1839f2");
        let date = NaiveDate::from_ymd(2014, 11, 28).and_hms_milli(7, 8, 9, 10);
        let expected_chunk = Chunk {
            chunk_identifier: identifier.clone(),
            expiration_date: date,
            root_handle: true,
        };

        let added_chunk = chunk_table.add_chunk(&identifier, date, true).expect("Chunk could not be added");
        assert_eq!(expected_chunk, added_chunk);
        
        expected_chunk
    }


    #[test]
    fn create_chunk_table() {
        _prepare_chunk_table("create_chunk_table");
    }

    #[test]
    fn add_chunk() {
        let chunk_table = _prepare_chunk_table("add_chunk");
        _prepare_one_chunk(&chunk_table);
    }

    #[test]
    fn remove_chunk() {
        let chunk_table = _prepare_chunk_table("remove_chunk");
        let expected_chunk = _prepare_one_chunk(&chunk_table);

        let removed = chunk_table.remove_chunk(&expected_chunk.chunk_identifier).expect("Could not remove chunk");
        assert_eq!(removed, 1);
    }

    #[test]
    fn get_chunk() {
        let chunk_table = _prepare_chunk_table("get_chunk");
        let expected_chunk = _prepare_one_chunk(&chunk_table);

        let got_chunk = chunk_table.get_chunk(&expected_chunk.chunk_identifier).expect("Could not remove chunk");
        assert_eq!(expected_chunk, got_chunk);
    }

    #[test]
    fn update_chunk() {
        let chunk_table = _prepare_chunk_table("update_chunk");
        let original_chunk = _prepare_one_chunk(&chunk_table);

        let date = NaiveDate::from_ymd(2015, 11, 28).and_hms_milli(7, 8, 9, 10);
        let expected_chunk = Chunk {
            chunk_identifier: original_chunk.chunk_identifier.clone(),
            expiration_date: date,
            root_handle: true,
        };

        let updated_chunk = chunk_table.update_chunk(&original_chunk.chunk_identifier, date, true).expect("Could not remove chunk");
        assert_eq!(expected_chunk, updated_chunk);
    }

    #[test]
    fn update_chunk_older_date() {
        let chunk_table = _prepare_chunk_table("update_chunk_older_date");
        let original_chunk = _prepare_one_chunk(&chunk_table);

        let second_date = NaiveDate::from_ymd(1970, 1, 1).and_hms_milli(7, 8, 9, 10);

        let updated_chunk = chunk_table.update_chunk(&original_chunk.chunk_identifier, second_date, false).expect("Could not remove chunk");
        assert_eq!(original_chunk, updated_chunk);
    }
}
