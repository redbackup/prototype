CREATE TABLE chunks (
    chunk_identifier TEXT NOT NULL PRIMARY KEY,
    expiration_date DATETIME NOT NULL,
    root_handle BOOLEAN NOT NULL
);
