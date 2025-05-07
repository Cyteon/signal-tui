pub fn init(db: &rusqlite::Connection) -> rusqlite::Result<()> {
    db.execute(
        "CREATE TABLE IF NOT EXISTS messages (
            id INTEGER PRIMARY KEY,
            sender TEXT NOT NULL,
            receiver TEXT NOT NULL,
            message TEXT NOT NULL,
            timestamp INTEGER NOT NULL
        )",
        [],
    )?;
    
    db.execute(
        "CREATE TABLE IF NOT EXISTS contacts (
            id STRING PRIMARY KEY,
            name TEXT NOT NULL,
            phoneNumber TEXT NOT NULL
        )",
        [],
    )?;

    db.execute(
        "CREATE TABLE IF NOT EXISTS groups (
            id STRING PRIMARY KEY,
            name TEXT NOT NULL,
            description TEXT NOT NULL,
            members TEXT NOT NULL
        )",
        [],
    )?;

    Ok(())
}