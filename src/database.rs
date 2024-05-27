use crate::models::{AnimalModel, BangPointModel};
use sqlx::{
    postgres::{PgPoolOptions, PgQueryResult},
    Error, Pool, Postgres,
};

pub struct Database {
    pub pool: Pool<Postgres>,
}

impl Database {
    pub async fn new(database_url: String) -> Result<Self, Error> {
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(database_url.as_str())
            .await?;

        Ok(Self { pool })
    }

    pub async fn add_animal(
        &self,
        animal: &str,
        emoji: &str,
        points: i32,
    ) -> Result<AnimalModel, Error> {
        sqlx::query_as!(
            AnimalModel,
            r#"INSERT INTO animals (animal, emoji, points) VALUES ($1, $2, $3) RETURNING id, animal, emoji, points"#,
            animal,
            emoji,
            points
        )
            .fetch_one(&self.pool)
            .await
    }

    pub async fn remove_animal(&self, animal: &str) -> Result<PgQueryResult, Error> {
        sqlx::query!("DELETE FROM animals WHERE animal = $1", animal)
            .execute(&self.pool)
            .await
    }

    pub async fn get_animal(&self, animal: &str) -> Result<AnimalModel, Error> {
        sqlx::query_as!(
            AnimalModel,
            r#"SELECT * FROM animals WHERE animal = $1"#,
            animal
        )
        .fetch_one(&self.pool)
        .await
    }

    pub async fn get_animals(&self) -> Result<Vec<AnimalModel>, Error> {
        sqlx::query_as!(AnimalModel, r#"SELECT * FROM animals"#)
            .fetch_all(&self.pool)
            .await
    }

    pub async fn get_bang_ranking(&self) -> Result<Vec<BangPointModel>, Error> {
        sqlx::query_as!(
            BangPointModel,
            r#"SELECT * FROM bang_points ORDER BY points LIMIT 10"#
        )
        .fetch_all(&self.pool)
        .await
    }
}
