use r2d2;
use diesel;
use r2d2::{Config,Pool};
use diesel::sqlite::SqliteConnection;
use r2d2_diesel::ConnectionManager;
use chrono::prelude::*;
use diesel::prelude::*;

mod schema;
#[cfg(test)] mod tests;

use self::schema::*;

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
    db_pool: Pool<ConnectionManager<SqliteConnection>>,
    file_name: String,
    creation_date: DateTime<Utc>
}

impl ChunkIndex {
    fn new(file_name: &str, creation_date: DateTime<Utc>) -> Result<Self, DatabaseError> {
        let config = Config::default();
        let manager = ConnectionManager::<SqliteConnection>::new(file_name);
        let db_pool = Pool::new(config, manager)?;

        let conn = db_pool.get()?;
        embedded_migrations::run(&*conn)?;

        Ok(ChunkIndex { db_pool, file_name: String::from(file_name), creation_date })
    }

    fn add_folder(&self, new_folder: NewFolder) -> Result<Folder,DatabaseError> {
        use self::folders::dsl;
        let conn = self.db_pool.get()?;
        diesel::insert(&new_folder).into(self::folders::table).execute(&*conn)?;

        let filtered_folders = dsl::folders.filter(dsl::name.eq(&new_folder.name));
        let folder = match new_folder.parent_folder {
            None => filtered_folders.first::<Folder>(&*conn)?,
            Some(id) => filtered_folders.filter(dsl::parent_folder.eq(id)).first::<Folder>(&*conn)?,
        };
        Ok(folder)
    }

    fn add_file(&self, new_file: NewFile) -> Result<File,DatabaseError> {
        use self::files::dsl;
        let conn = self.db_pool.get()?;
        diesel::insert(&new_file).into(self::files::table).execute(&*conn)?;

        let file = dsl::files.filter(dsl::name.eq(&new_file.name))
            .filter(dsl::folder.eq(new_file.folder)).first::<File>(&*conn)?;
        Ok(file)
    }

    fn add_chunk(&self, new_chunk: NewChunk) -> Result<Chunk,DatabaseError> {
        use self::chunks::dsl;
        let conn = self.db_pool.get()?;
        diesel::insert(&new_chunk).into(self::chunks::table).execute(&*conn)?;

        let chunk = dsl::chunks.filter(dsl::chunk_identifier.eq(&new_chunk.chunk_identifier))
            .filter(dsl::file.eq(new_chunk.file)).first::<Chunk>(&*conn)?;
        Ok(chunk)
    }
}
