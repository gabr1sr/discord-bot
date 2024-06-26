#[derive(Default, Debug, sqlx::FromRow)]
pub struct Tag {
    pub id: i64,
    pub name: String,
    pub content: String,
    pub owner: String,
}
