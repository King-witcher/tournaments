//! Para quem e sobre o que um torneio avisa no tempo real.

use sqlx::PgConnection;

use super::TournamentId;
use crate::tenants::TenantId;

/// O endereço do torneio (slugs da comunidade e dele), o nome e os inscritos.
pub struct Audience {
    pub tenant_id: TenantId,
    pub tenant: String,
    pub slug: String,
    /// No idioma original.
    pub name: String,
    pub participants: Vec<i64>,
    /// Na inscrição ou no check-in com o início marcado já passado (agora, na
    /// transação): espera a organização iniciar (`GET /me/organizing`).
    pub start_due: bool,
}

impl Audience {
    /// Caminho do torneio no site, sem o idioma.
    pub fn path(&self) -> String {
        format!("/{}/{}", self.tenant, self.slug)
    }
}

pub async fn audience(conn: &mut PgConnection, id: TournamentId) -> Result<Audience, sqlx::Error> {
    let row = sqlx::query!(
        r#"select t.tenant_id, te.slug as tenant, t.slug, t.name,
                  coalesce(array_agg(p.user_id order by p.user_id)
                               filter (where p.user_id is not null),
                           '{}') as "participants!",
                  (t.status in ('registration', 'check_in')
                   and coalesce(t.starts_at <= now(), false)) as "start_due!"
           from tournaments t
           join tenants te on te.id = t.tenant_id
           left join participants p on p.tournament_id = t.id
           where t.id = $1
           group by t.id, te.slug"#,
        id.get()
    )
    .fetch_one(conn)
    .await?;
    Ok(Audience {
        tenant_id: TenantId::new(row.tenant_id),
        tenant: row.tenant,
        slug: row.slug,
        name: row.name,
        participants: row.participants,
        start_due: row.start_due,
    })
}
