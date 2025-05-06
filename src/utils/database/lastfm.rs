use std::any::Any;

pub async fn prepare_lastfm_db(db_client: &async_sqlite::Client) {
    db_client.conn(|conn: &async_sqlite::rusqlite::Connection| {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS lastfm_sessions (
                discord_id INTEGER PRIMARY KEY,
                session_key TEXT
            )",
            []
        )
    }).await.expect("Failed to create lastfm_sessions table in Loopchan's Database");
}

pub async fn save_lastfm_session_key(
    db_client: &async_sqlite::Client,
    discord_id: u64,
    session_key: String
) -> Result<usize, async_sqlite::Error> {
    db_client.conn(move |conn: &async_sqlite::rusqlite::Connection| {
        conn.execute(
            "INSERT INTO lastfm_sessions (discord_id, session_key) VALUES (?1, ?2) ON CONFLICT DO UPDATE SET session_key = EXCLUDED.session_key",
            (discord_id, session_key)
        )
    }).await
}

pub async fn get_lastfm_session_key(
    db_client: &async_sqlite::Client,
    discord_id: u64
) -> Result<Option<String>, async_sqlite::Error> {
    let result: Result<Option<String>, async_sqlite::Error> = db_client.conn(move |conn: &async_sqlite::rusqlite::Connection| {
        conn.query_row(
            "SELECT session_key FROM lastfm_sessions WHERE discord_id=?",
            [discord_id],
            |row| row.get(0),
        )
    }).await;
    if result.is_err() {
        let result_err: async_sqlite::Error = result.unwrap_err();
        if result_err.type_id() == async_sqlite::Error::Rusqlite(async_sqlite::rusqlite::Error::QueryReturnedNoRows).type_id() {
            return Ok(None);
        }
        return Err(result_err);
    }
    return result;
}