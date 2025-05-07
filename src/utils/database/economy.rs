use tracing::error;

pub async fn prepare_eco_db(db_client: &async_sqlite::Client) {
    db_client.conn(|conn: &async_sqlite::rusqlite::Connection| {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS economics (
                discord_id INTEGER PRIMARY KEY,
                balance INTEGER,
                level INTEGER,
                experience INTEGER
            )",
            []
        )
    }).await.expect("Failed to create economics table in Loopchan's Database");
}

pub async fn create_user_in_eco_db(
    db_client: &async_sqlite::Client,
    discord_id: u64
) -> Result<usize, async_sqlite::Error> {
    db_client.conn(move |conn: &async_sqlite::rusqlite::Connection| {
        conn.execute(
            "INSERT INTO economics (discord_id, balance, level, experience) VALUES (?1, ?2, ?3, ?4) ON CONFLICT DO NOTHING",
            (discord_id, 0, 1, 0)
        )
    }).await
}

pub async fn get_user_balance_in_eco_db(
    db_client: &async_sqlite::Client,
    discord_id: u64
) -> Result<u64, async_sqlite::Error> {
    db_client.conn(move |conn: &async_sqlite::rusqlite::Connection| {
        conn.query_row(
            "SELECT balance FROM economics WHERE discord_id=?",
            [discord_id],
            |row| row.get(0),
        )
    }).await
}

pub async fn get_user_level_and_experience_in_eco_db(
    db_client: &async_sqlite::Client,
    discord_id: u64
) -> Result<(Result<u64, async_sqlite::rusqlite::Error>, Result<u64, async_sqlite::rusqlite::Error>), async_sqlite::Error> {
    db_client.conn(move |conn: &async_sqlite::rusqlite::Connection| {
        conn.query_row(
            "SELECT level, experience FROM economics WHERE discord_id=?",
            [discord_id],
            |row| Ok((row.get(0), row.get(1))),
        )
    }).await
}

pub async fn build_level_leaderboard_from_eco_db(db_client: &async_sqlite::Client) -> Result<Vec<(u64, u64, u64)>, async_sqlite::Error> {
    db_client.conn(move |conn: &async_sqlite::rusqlite::Connection| {
        let query = format!(
            "SELECT discord_id, level, experience FROM economics ORDER BY level DESC, experience DESC LIMIT 5",
        );
    
        let mut binding = conn.prepare(&query).unwrap();
        let mut rows = binding.query([]).unwrap();
        let mut leaderboard = Vec::new();
    
        while let Ok(Some(row)) = rows.next() {
            let discord_id: u64 = row.get(0)?;
            let level: u64 = row.get(1)?;
            let experience: u64 = row.get(2)?;
            leaderboard.push((discord_id, level, experience));
        }
    
        Ok(leaderboard)
    }).await
}

pub async fn get_user_placement_in_level_leaderboard(
    db_client: &async_sqlite::Client,
    discord_id: u64
)-> Result<u8, async_sqlite::Error> {
    db_client.conn(move |conn: &async_sqlite::rusqlite::Connection| {
        conn.query_row(
            "SELECT * FROM ( SELECT ROW_NUMBER() OVER (ORDER BY level DESC, experience DESC) AS RowNum, * FROM economics ) AS RowResults WHERE discord_id=?",
            [discord_id],
            |row| row.get(0),
        )
    }).await
}

pub async fn build_balance_leaderboard_from_eco_db(db_client: &async_sqlite::Client) -> Result<Vec<(u64, u64)>, async_sqlite::Error> {
    db_client.conn(move |conn: &async_sqlite::rusqlite::Connection| {
        let query = format!(
            "SELECT discord_id, balance FROM economics ORDER BY balance DESC LIMIT 5",
        );
    
        let mut binding = conn.prepare(&query).unwrap();
        let mut rows = binding.query([]).unwrap();
        let mut leaderboard = Vec::new();
    
        while let Ok(Some(row)) = rows.next() {
            let discord_id: u64 = row.get(0)?;
            let balance: u64 = row.get(1)?;
            leaderboard.push((discord_id, balance));
        }
    
        Ok(leaderboard)
    }).await
}

pub async fn get_user_placement_in_balance_leaderboard(
    db_client: &async_sqlite::Client,
    discord_id: u64
)-> Result<u8, async_sqlite::Error> {
    db_client.conn(move |conn: &async_sqlite::rusqlite::Connection| {
        conn.query_row(
            "SELECT * FROM ( SELECT ROW_NUMBER() OVER (ORDER BY balance DESC) AS RowNum, * FROM economics ) AS RowResults WHERE discord_id=?",
            [discord_id],
            |row| row.get(0),
        )
    }).await
}

pub async fn update_user_level_and_experience_in_eco_db(
    db_client: &async_sqlite::Client,
    discord_id: u64,
    level: Option<u64>,
    experience: Option<u64>
) -> Result<usize, async_sqlite::Error> {
    // Update both level and experience
    if !level.is_none() && !experience.is_none() {
        return db_client.conn(move |conn: &async_sqlite::rusqlite::Connection| {
            conn.execute(
                "UPDATE economics SET level=(?2), experience=(?3) WHERE discord_id=(?1)",
                (discord_id, level, experience)
            )
        }).await;
    }

    // Update only level
    if !level.is_none() {
        return db_client.conn(move |conn: &async_sqlite::rusqlite::Connection| {
            conn.execute(
                "UPDATE economics SET level=(?2) WHERE discord_id=(?1)",
                (discord_id, level)
            )
        }).await;
    }

    if experience.is_none() {
        error!("Both level and experience weren't provided to `update_user_level_and_experience_in_eco_db`!");
        return Ok(0);
    }

    // Update only experience
    return db_client.conn(move |conn: &async_sqlite::rusqlite::Connection| {
        conn.execute(
            "UPDATE economics SET experience=(?2) WHERE discord_id=(?1)",
            (discord_id, experience)
        )
    }).await;
}