use sqlx::postgres::{PgPool, PgPoolOptions};

#[derive(Debug, Clone)]
pub struct Db {
    pool: PgPool,
}

impl Db {
    pub async fn initialize(uri: &str) -> Result<Self, sqlx::Error> {
        // Create a connection pool
        let pool = PgPoolOptions::new().max_connections(5).connect(uri).await?;

        Ok(Db { pool })
    }

    pub fn database(&self) -> &PgPool {
        &self.pool
    }
}
