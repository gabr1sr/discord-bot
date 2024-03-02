use sqlx::{
    Error,
    postgres::PgPoolOptions, Pool, Postgres
};

pub struct Database {
    pub pool: Pool<Postgres>,
}

impl Database {
    pub async fn new(
        database_url: String
    ) -> Result<Self, Error> {
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(database_url.as_str()).await?;
        
        Ok(Self { pool })
    }
}
