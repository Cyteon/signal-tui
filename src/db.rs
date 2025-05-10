pub fn init(db: &rusqlite::Connection) -> rusqlite::Result<()> {
    db.execute(
        "CREATE TABLE IF NOT EXISTS messages (
            id INTEGER PRIMARY KEY,
            sender TEXT NOT NULL,
            receiver TEXT NOT NULL,
            message TEXT NOT NULL,
            timestamp INTEGER NOT NULL,
            accountNumber TEXT NOT NULL
        )",
        [],
    )?;
    
    db.execute(
        "CREATE TABLE IF NOT EXISTS contacts (
            id STRING PRIMARY KEY,
            name TEXT NOT NULL,
            phoneNumber TEXT NOT NULL,
            accountNumber TEXT NOT NULL
        )",
        [],
    )?;

    /*
        pub id: String,
        pub name: String,
        pub description: String,
        pub is_member: bool,
        pub is_blocked: bool,
        pub members: Vec<String>,
        pub pending_members: Vec<String>,
        pub requesting_members: Vec<String>,
        pub admins: Vec<String>,
        pub group_invite_link: String
    */

    db.execute(
        "CREATE TABLE IF NOT EXISTS groups (
            id STRING PRIMARY KEY,
            name TEXT NOT NULL,
            description TEXT NOT NULL,
            is_member BOOLEAN NOT NULL,
            is_blocked BOOLEAN NOT NULL,
            members TEXT NOT NULL,
            pending_members TEXT NOT NULL,
            requesting_members TEXT NOT NULL,
            admins TEXT NOT NULL,
            group_invite_link TEXT NOT NULL,
            accountNumber TEXT NOT NULL
        )",
        [],
    )?;

    Ok(())
}