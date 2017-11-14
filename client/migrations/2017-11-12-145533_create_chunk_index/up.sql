CREATE TABLE folders (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    name TEXT NOT NULL,
    parent_folder INTEGER,
    FOREIGN KEY(parent_folder) REFERENCES folders(id)
);

CREATE TABLE files (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    name TEXT NOT NULL,
    last_change_date DATETIME NOT NULL,
    folder INTEGER NOT NULL,
    FOREIGN KEY(folder) REFERENCES folders(id)
);

CREATE TABLE chunks (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    chunk_identifier TEXT NOT NULL,
    file INTEGER NOT NULL,
    predecessor INTEGER,
    UNIQUE(file,chunk_identifier),
    FOREIGN KEY(file) REFERENCES files(id),
    FOREIGN KEY(predecessor) REFERENCES chunks(id)
);
