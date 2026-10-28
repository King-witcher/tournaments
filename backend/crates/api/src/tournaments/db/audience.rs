//! Para quem e sobre o que um torneio avisa no tempo real.

use sqlx::PgConnection;

use super::TournamentId;

/// O endereço do torneio (slugs da comunidade e dele), o nome e os inscritos.
pub struct Audience {
    pub tenant: String,
    pub slug: String,
    /// No idioma original.
    pub name: String,
    pub participants: Vec<i64>,
}

impl Audience {
    /// Caminho do torneio no site, sem o idioma.
    pub fn path(&self) -> String {
        format!("/{}/{}", self.tenant, self.slug)
    }
}

pub async fn audience(conn: &mut PgConnection, id: TournamentId) -> Result<Audience, sqlx::Error> {
    sqlx::query_as!(
        Audience,
        r#"select te.slug as tenant, t.slug, t.name,
                  coalesce(array_agg(p.user_id order by p.user_id)
                               filter (where p.user_id is not null),
                           '{}') as "participants!"
           from tournaments t
           join tenants te on te.id = t.tenant_id
           left join participants p on p.tournament_id = t.id
           where t.id = $1
           group by te.slug, t.slug, t.name"#,
        id.get()
    )
    .fetch_one(conn)
    .await
}
