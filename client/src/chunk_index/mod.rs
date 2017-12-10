use std::path::PathBuf;

use r2d2;
use diesel;
use r2d2::{Config, Pool, PooledConnection};
use diesel::sqlite::SqliteConnection;
use r2d2_diesel::ConnectionManager;
use chrono::prelude::*;
use diesel::prelude::*;

pub mod schema;

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

/// Client chunk index, to store the local folder and file structures and hashes:
#[derive(Clone)]
pub struct ChunkIndex {
    db_pool: Pool<ConnectionManager<SqliteConnection>>,
    file_name: PathBuf,
    creation_date: DateTime<Utc>,
}

impl ChunkIndex {
    pub fn new(file_name: PathBuf, creation_date: DateTime<Utc>) -> Result<Self, DatabaseError> {
        debug!("Connect to Database {:?}", file_name);
        let config = Config::default();
        let manager = ConnectionManager::<SqliteConnection>::new(file_name.to_string_lossy());
        let db_pool = Pool::new(config, manager)?;

        let conn = db_pool.get()?;
        debug!("Run Database migrations");
        embedded_migrations::run(&*conn)?;

        debug!("Finished creating chunk index");
        Ok(ChunkIndex {
            db_pool,
            file_name,
            creation_date,
        })
    }

    pub fn get_file_name(&self) -> PathBuf {
        self.file_name.clone()
    }

    pub fn get_db_connection(
        &self,
    ) -> Result<PooledConnection<ConnectionManager<SqliteConnection>>, DatabaseError> {
        let conn = self.db_pool.get()?;
        Ok(conn)
    }

    pub fn add_folder(&self, new_folder: NewFolder) -> Result<Folder, DatabaseError> {
        use self::folders::dsl;
        let conn = self.get_db_connection()?;
        diesel::insert(&new_folder)
            .into(self::folders::table)
            .execute(&*conn)?;

        // sqlite does not support RETURNING clauses, so query the new folder from database
        let query_folder_name = dsl::folders.filter(dsl::name.eq(&new_folder.name));
        let folder;
        if let Some(id) = new_folder.parent_folder {
            folder = query_folder_name
                .filter(dsl::parent_folder.eq(id))
                .first::<Folder>(&*conn);
        } else {
            folder = query_folder_name
                .filter(dsl::parent_folder.is_null())
                .first::<Folder>(&*conn);
        }
        folder.map_err(|e| DatabaseError::from(e))
    }

    pub fn add_file(&self, new_file: NewFile) -> Result<File, DatabaseError> {
        use self::files::dsl;
        let conn = self.get_db_connection()?;
        diesel::insert(&new_file).into(self::files::table).execute(
            &*conn,
        )?;

        // sqlite does not support RETURNING clauses, so query the new file from database
        let file = dsl::files
            .filter(dsl::name.eq(&new_file.name))
            .filter(dsl::folder.eq(new_file.folder))
            .first::<File>(&*conn)?;
        Ok(file)
    }

    pub fn add_chunk(&self, new_chunk: NewChunk) -> Result<Chunk, DatabaseError> {
        use self::chunks::dsl;
        let conn = self.get_db_connection()?;
        diesel::insert(&new_chunk)
            .into(self::chunks::table)
            .execute(&*conn)?;

        // sqlite does not support RETURNING clauses, so query the new chunks from database
        let chunk = dsl::chunks
            .filter(dsl::chunk_identifier.eq(&new_chunk.chunk_identifier))
            .filter(dsl::file.eq(new_chunk.file))
            .first::<Chunk>(&*conn)?;
        Ok(chunk)
    }

    pub fn get_all_chunks(&self) -> Result<Vec<Chunk>, DatabaseError> {
        let conn = self.get_db_connection()?;
        self::chunks::table.load(&*conn).map_err(
            |e| DatabaseError::from(e),
        )
    }

    /// Get the relative path of a file by id.
    pub fn get_file_path(&self, file_id: i32) -> Result<PathBuf, DatabaseError> {
        use self::folders::dsl;
        use self::files;
        let conn = self.get_db_connection()?;

        conn.transaction::<_, DatabaseError, _>(|| {

            let file = files::dsl::files
                .filter(files::dsl::id.eq(&file_id))
                .first::<File>(&*conn)?;
            let mut parent_id = file.folder;
            let mut path_vec = vec![file.name.clone()];
            let mut path = PathBuf::new();

            // query parent folders recursively
            loop {
                let folder = dsl::folders.filter(dsl::id.eq(parent_id)).first::<Folder>(
                    &*conn,
                )?;
                path_vec.push(folder.name.clone());

                if let Some(parent) = folder.parent_folder {
                    parent_id = parent;
                } else {
                    break;
                }
            }

            path_vec.reverse();
            path_vec.iter().for_each(|e| path.push(e));
            Ok(path)
        })
    }

    /// Get the subfolder of Some folder, or all root folders with None.
    pub fn get_folders_by_parent(
        &self,
        parent_folder: Option<i32>,
    ) -> Result<Vec<Folder>, DatabaseError> {
        use self::folders::dsl;
        let conn = self.get_db_connection()?;
        if let Some(parent_folder) = parent_folder {
            dsl::folders
                .filter(dsl::parent_folder.eq(parent_folder))
                .load::<Folder>(&*conn)
                .map_err(|e| DatabaseError::from(e))
        } else {
            dsl::folders
                .filter(dsl::parent_folder.is_null())
                .load::<Folder>(&*conn)
                .map_err(|e| DatabaseError::from(e))
        }
    }
}
