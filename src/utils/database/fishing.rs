use crate::DataFish;

pub async fn prepare_fishing_db(db_client: &async_sqlite::Client) {
    db_client.conn(|conn: &async_sqlite::rusqlite::Connection| {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS fishes (
                discord_id INTEGER,
                uuid TEXT PRIMARY KEY,
                type TEXT,
                modifiers TEXT,
                size REAL
            )",
            []
        )
    }).await.expect("Failed to create fishing table in Loopchan's Database");
}

pub async fn give_fish_to_user_in_fishing_db(
    db_client: &async_sqlite::Client,
    discord_id: u64,
    fish: DataFish
) -> Result<usize, async_sqlite::Error> {
    db_client.conn(move |conn: &async_sqlite::rusqlite::Connection| {
        conn.execute(
            "INSERT INTO fishes (discord_id, uuid, type, modifiers, size) VALUES (?, ?, ?, ?, ?) ON CONFLICT DO NOTHING",
            (discord_id, fish.uuid, fish.r#type, fish.modifiers, fish.size)
        )
    }).await
}

const GET_USER_FISHES_QUERY: &str = "SELECT uuid, type, modifiers, size FROM fishes WHERE discord_id = ?";

pub async fn get_user_fishes_in_fishing_db(
    db_client: &async_sqlite::Client,
    discord_id: u64
) -> Result<Vec<DataFish>, async_sqlite::Error> {
    db_client.conn(move |conn: &async_sqlite::rusqlite::Connection| {
        let mut stmt = conn.prepare(GET_USER_FISHES_QUERY).unwrap();
        let mut rows = stmt.query([discord_id]).unwrap();
        let mut inventory = Vec::new();

        while let Ok(Some(row)) = rows.next() {
            inventory.push(
                DataFish {
                    uuid: row.get(0)?,
                    r#type: row.get(1)?,
                    modifiers: row.get(2)?,
                    size: row.get(3)?,
                }
            );
        }
    
        Ok(inventory)
    }).await
}