//! Traduções de um torneio.

use sqlx::PgConnection;

use super::TournamentId;
use crate::{shared::Locale, tournaments::texts::Texts};

/// Cria ou substitui a tradução de `locale`.
pub async fn put_translation(
    conn: &mut PgConnection,
    id: TournamentId,
    locale: Locale,
    texts: &Texts,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"insert into tournament_translations (tournament_id, locale, name, description, rules)
           values ($1, $2, $3, $4, $5)
           on conflict (tournament_id, locale) do update
           set name = excluded.name, description = excluded.description, rules = excluded.rules"#,
        id.get(),
        locale.as_str(),
        texts.name,
        texts.description,
        texts.rules
    )
    .execute(conn)
    .await?;
    Ok(())
}

/// `false` se não havia tradução nesse idioma.
pub async fn delete_translation(
    conn: &mut PgConnection,
    id: TournamentId,
    locale: Locale,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query!(
        "delete from tournament_translations where tournament_id = $1 and locale = $2",
        id.get(),
        locale.as_str()
    )
    .execute(conn)
    .await?;
    Ok(result.rows_affected() > 0)
}
