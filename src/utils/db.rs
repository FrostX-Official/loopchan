use super::basic::{check_env, parse_env_as_string};
use tracing::warn;

const DEFAULT_DATABASE_PATH: &'static str = "loopchan.db";

pub async fn create_db() -> Result<async_sqlite::Client, async_sqlite::Error> {
    if check_env("DATABASE_PATH") {
        return async_sqlite::ClientBuilder::new()
            .path(parse_env_as_string("DATABASE_PATH"))
            .open()
            .await;
    }

    async_sqlite::ClientBuilder::new()
        .path(DEFAULT_DATABASE_PATH)
        .open()
        .await
}

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

pub async fn create_user_in_users_db(
    db_client: &async_sqlite::Client,
    discord_id: u64,
    roblox_id: u64
) -> Result<usize, async_sqlite::Error> {
    // Would be nice if I knew how to cache recently created users to not create a lot of db calls! (hopefully it isn't that expensive as I think it is.)
    db_client.conn(move |conn: &async_sqlite::rusqlite::Connection| {
        conn.execute(
            "INSERT INTO users (discord_id, roblox_id) VALUES (?1, ?2) ON CONFLICT DO NOTHING",
            (discord_id, roblox_id)
        )
    }).await
}

pub async fn create_user_in_eco_db(
    db_client: &async_sqlite::Client,
    discord_id: u64
) -> Result<usize, async_sqlite::Error> {
    // Would be nice if I knew how to cache recently created users to not create a lot of db calls! (hopefully it isn't that expensive as I think it is.)
    db_client.conn(move |conn: &async_sqlite::rusqlite::Connection| {
        conn.execute(
            "INSERT INTO economics (discord_id, balance, level, experience) VALUES (?1, ?2, ?3, ?4) ON CONFLICT DO NOTHING",
            (discord_id, 0, 1, 0)
        )
    }).await
}

pub async fn update_roblox_id_in_users_db(
    db_client: &async_sqlite::Client,
    discord_id: u64,
    roblox_id: u64
) -> Result<usize, async_sqlite::Error> {
    db_client.conn(move |conn: &async_sqlite::rusqlite::Connection| {
        conn.execute(
            "UPDATE users SET roblox_id=(?1) WHERE discord_id=(?2)",
            (roblox_id, discord_id)
        )
    }).await
}

// I'll change this someday
pub async fn get_roblox_id_in_users_db_by_discord_id(
    db_client: &async_sqlite::Client,
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

pub async fn _get_discord_id_in_users_db_by_roblox_id( // TODO: Made for pstats command
    db_client: &async_sqlite::Client,
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
        warn!("Both level and experience weren't provided to `update_user_level_and_experience_in_eco_db`!");
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