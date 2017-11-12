CREATE TABLE snapshots (
    uuid TEXT NOT NULL PRIMARY KEY,
    creation_date DATETIME NOT NULL,
    expiration_date DATETIME NOT NULL
);

CREATE TABLE chunks (
    file_name TEXT NOT NULL,
    chunk_identifier TEXT NOT NULL,
    PRIMARY KEY(file_name,chunk_identifier)
);

CREATE TABLE snapshotchunks (
    snapshot_uuid TEXT NOT NULL,
    file_name TEXT NOT NULL,
    chunk_identifier TEXT NOT NULL,
    PRIMARY KEY(snapshot_uuid, chunk_identifier, file_name),
    FOREIGN KEY(snapshot_uuid) REFERENCES snapshots(uuid),
    FOREIGN KEY(snapshot_uuid, file_name) REFERENCES snapshots(uuid,file_name)
);
