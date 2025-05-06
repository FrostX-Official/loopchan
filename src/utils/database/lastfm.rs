pub async fn prepare_lastfm_db(db_client: &async_sqlite::Client) {
    db_client.conn(|conn: &async_sqlite::rusqlite::Connection| {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS lastfm_sessions (
                discord_id INTEGER PRIMARY KEY,
                session_key TEXT,
                username TEXT
            )",
            []
        )
    }).await.expect("Failed to create lastfm_sessions table in Loopchan's Database");
}

pub async fn save_lastfm_session_data(
    db_client: &async_sqlite::Client,
    discord_id: u64,
    session_key: String,
    username: String
) -> Result<usize, async_sqlite::Error> {
    db_client.conn(move |conn: &async_sqlite::rusqlite::Connection| {
        conn.execute(
            "INSERT INTO lastfm_sessions (discord_id, session_key, username) VALUES (?1, ?2, ?3) ON CONFLICT DO UPDATE SET session_key = EXCLUDED.session_key, username = EXCLUDED.username",
            (discord_id, session_key, username)
        )
    }).await
}

pub async fn get_lastfm_session_data(
    db_client: &async_sqlite::Client,
    discord_id: u64
) -> Result<(String, String), async_sqlite::Error> {
    db_client.conn(move |conn: &async_sqlite::rusqlite::Connection| {
        conn.query_row(
            "SELECT session_key, username FROM lastfm_sessions WHERE discord_id=?",
            [discord_id],
            |row| Ok((row.get(0).unwrap(), row.get(1).unwrap())),
        )
    }).await
}