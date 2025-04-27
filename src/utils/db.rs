pub async fn prepare_users_db(db_client: &async_sqlite::Client) {
    db_client.conn(|conn: &async_sqlite::rusqlite::Connection| {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS users (
                discord_id INTEGER PRIMARY KEY,
                roblox_id INTEGER,
                staff BOOLEAN,
                qa BOOLEAN
            )",
            []
        )
    }).await.expect("Failed to create users table in Loopchan's Database");
}

pub async fn create_user_in_db(
    db_client: &async_sqlite::Client,
    author_id: u64,
    roblox_id: u64,
    is_staff: bool,
    is_qa: bool
)-> Result<usize, async_sqlite::Error> {
    db_client.conn(move |conn| {
        conn.execute(
            "INSERT INTO users (discord_id, roblox_id, staff, qa) VALUES (?1, ?2, ?3, ?4)",// ON CONFLICT DO NOTHING
            (author_id, roblox_id, is_staff, is_qa)
        )
    }).await
}