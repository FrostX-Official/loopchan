// Would be nice if I knew how to cache recently created users to not create a lot of db calls! (hopefully it isn't that expensive as I think it is.)

pub mod linking;
pub mod economy;
pub mod lastfm;

const DEFAULT_DATABASE_PATH: &'static str = "loopchan.db";

pub async fn create_db(path: Option<String>) -> Result<async_sqlite::Client, async_sqlite::Error> {
    if path.is_some() {
        return async_sqlite::ClientBuilder::new()
            .path(path.unwrap())
            .open()
            .await;
    }

    async_sqlite::ClientBuilder::new()
        .path(DEFAULT_DATABASE_PATH)
        .open()
        .await
}