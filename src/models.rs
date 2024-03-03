use sqlx::types::chrono::{self, Local};

#[derive(Clone, Debug, PartialEq, PartialOrd, sqlx::Type, poise::ChoiceParameter)]
#[sqlx(type_name = "severity", rename_all = "lowercase")]
pub enum Severity {
    Low,
    Mid,
    High,
}

#[derive(Clone, Debug, PartialEq, PartialOrd, sqlx::Type, poise::ChoiceParameter)]
#[sqlx(type_name = "punishment", rename_all = "lowercase")]
pub enum Punishment {
    Strike,
    Timeout,
    Ban,
}

#[derive(Debug, sqlx::FromRow)]
pub struct InfractionModel {
    pub id: i32,
    pub severity: Severity,
    pub punishment: Punishment,
    pub duration: i32,
}

#[derive(Debug, sqlx::FromRow)]
pub struct PunishmentModel {
    pub id: i32,
    pub user_id: String,
    pub punishment: Punishment,
    pub duration: i32,
}

#[derive(Debug, sqlx::FromRow)]
pub struct UserInfractionModel {
    pub id: i32,
    pub user_id: String,
    pub infraction_id: i32,
    pub created_at: chrono::DateTime<Local>,
}
