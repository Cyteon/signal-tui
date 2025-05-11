pub fn init(db: &rusqlite::Connection) -> rusqlite::Result<()> {
    db.execute(
        "CREATE TABLE IF NOT EXISTS messages (
            id TEXT PRIMARY KEY,
            sourceUuid TEXT NOT NULL,
            sourceNumber TEXT,
            sourceName TEXT NOT NULL,
            destinationUuid TEXT,
            groupId TEXT,
            message TEXT NOT NULL,
            timestamp INTEGER NOT NULL,
            expiresInSeconds INTEGER,
            pending INTEGER NOT NULL,
            accountNumber TEXT
        )",
        [],
    )?;

    Ok(())
}