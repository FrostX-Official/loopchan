pub async fn prepare_users_db(db_client: &async_sqlite::Client) {
    db_client.conn(|conn: &async_sqlite::rusqlite::Connection| {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS users (
                discord_id INTEGER PRIMARY KEY,
                roblox_id INTEGER
            )",
            []
        )
    }).await.expect("Failed to create users table in Loopchan's Database");
}

pub async fn create_user_in_db(
    db_client: &async_sqlite::Client,
    author_id: u64,
    roblox_id: u64
) -> Result<usize, async_sqlite::Error> {
    // Would be nice if I knew how to cache recently created users to not create a lot of db calls! (hopefully it isn't that expensive as I think it is.)
    db_client.conn(move |conn: &async_sqlite::rusqlite::Connection| {
        conn.execute(
            "INSERT INTO users (discord_id, roblox_id) VALUES (?1, ?2) ON CONFLICT DO NOTHING",
            (author_id, roblox_id)
        )
    }).await
}

pub async fn get_roblox_id_in_db_by_discord_id(db_client: &async_sqlite::Client,
    discord_id: u64
) -> Result<u64, async_sqlite::Error> {
    db_client.conn(move |conn: &async_sqlite::rusqlite::Connection| {
        conn.query_row(
            "SELECT roblox_id FROM users WHERE discord_id=?",
            [discord_id],
            |row| row.get(0),
        )
    }).await
}

pub async fn _get_discord_id_in_db_by_roblox_id(db_client: &async_sqlite::Client, // TODO: Made for pstats command
    roblox_id: u64
) -> Result<u64, async_sqlite::Error> {
    db_client.conn(move |conn: &async_sqlite::rusqlite::Connection| {
        conn.query_row(
            "SELECT discord_id FROM users WHERE roblox_id=?",
            [roblox_id],
            |row| row.get(0),
        )
    }).await
}