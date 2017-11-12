CREATE TABLE snapshots (
    uuid TEXT NOT NULL PRIMARY KEY,
    creation_date DATETIME NOT NULL
);

CREATE TABLE chunks (
    file TEXT NOT NULL,
    chunk_identifier TEXT NOT NULL,
    expiration_date DATETIME NOT NULL,
    snapshot TEXT NOT NULL,
    PRIMARY KEY(file,chunk_identifier),
    FOREIGN KEY(snapshot) REFERENCES snapshots(uuid)
);
