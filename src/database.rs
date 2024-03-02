use tokio_postgres::{Client, Error, NoTls};

pub struct Database {
    pub client: Client,
}

impl Database {
    pub async fn new(
        host: String,
        port: String,
        user: String,
        password: String,
    ) -> Result<Self, Error> {
        let config = format!("host={host} port={port} user={user} password={password}");
        let (client, connection) = tokio_postgres::connect(config.as_str(), NoTls).await?;

        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("connection error: {}", e);
            }
        });

        Ok(Self { client })
    }
}
