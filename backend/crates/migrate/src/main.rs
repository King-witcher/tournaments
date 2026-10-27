//! Aplica as migrations de `backend/migrations` no banco de `DATABASE_URL`.
//!
//! Existe porque as macros `sqlx::query!` conferem o SQL contra o banco na
//! compilação: uma tabela nova precisa existir antes de compilar a API que a
//! usa. Este binário não usa essas macros, então sempre compila.

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    let url = std::env::var("DATABASE_URL")?;
    let pool = sqlx::PgPool::connect(&url).await?;

    let migrator = sqlx::migrate!("../../migrations");
    migrator.run(&pool).await?;

    let applied = migrator.iter().count();
    println!("{applied} migrations em dia.");
    Ok(())
}
