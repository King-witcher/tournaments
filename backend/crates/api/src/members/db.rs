use sqlx::{PgConnection, PgPool};

use super::types::{JoinedVia, Member};
use crate::{
    shared::ApiError,
    tenants::{Role, TenantId},
};

/// Membros ativos do tenant (ou só `only_user`), owner primeiro, depois
/// organizers e players, cada grupo por apelido. `with_origin` preenche
/// `joined_via` (só para quem organiza; para os outros fica nulo).
pub async fn list(
    db: &PgPool,
    tenant: TenantId,
    only_user: Option<i64>,
    with_origin: bool,
) -> Result<Vec<Member>, ApiError> {
    let rows = sqlx::query!(
        r#"select u.id as user_id, m.nickname, u.display_name, u.avatar_url, m.role, m.joined_at,
                  i.id is not null as "has_invite!", i.token_hint, t.name as "tournament_name?"
           from tenant_members m
           join users u on u.id = m.user_id
           left join tenant_invites i on i.tenant_id = m.tenant_id and i.id = m.invite_id
           left join tournaments t on t.tenant_id = i.tenant_id and t.id = i.tournament_id
           where m.tenant_id = $1 and m.left_at is null
             and ($2::bigint is null or m.user_id = $2)
           order by case m.role when 'owner' then 0 when 'organizer' then 1 else 2 end,
                    lower(m.nickname), m.user_id"#,
        tenant.get(),
        only_user
    )
    .fetch_all(db)
    .await?;

    rows.into_iter()
        .map(|r| {
            Ok(Member {
                user_id: r.user_id,
                nickname: r.nickname,
                display_name: r.display_name,
                avatar_url: r.avatar_url,
                role: Role::from_db(&r.role)?,
                joined_at: r.joined_at,
                joined_via: (with_origin && r.has_invite).then_some(JoinedVia {
                    hint: r.token_hint,
                    tournament: r.tournament_name,
                }),
            })
        })
        .collect()
}

/// Papel do usuário no tenant; `None` se ele não é membro ativo.
pub async fn role_of(
    db: &PgPool,
    tenant: TenantId,
    user_id: i64,
) -> Result<Option<Role>, ApiError> {
    sqlx::query_scalar!(
        r#"select role from tenant_members
           where tenant_id = $1 and user_id = $2 and left_at is null"#,
        tenant.get(),
        user_id
    )
    .fetch_optional(db)
    .await?
    .map(|role| Role::from_db(&role))
    .transpose()
}

/// Como `role_of`, travando a linha do membro até o fim da transação (sem
/// travar quem só a referencia, como uma inscrição nova): papel e
/// participação não mudam até a saída gravar.
pub async fn lock_role(
    conn: &mut PgConnection,
    tenant: TenantId,
    user_id: i64,
) -> Result<Option<Role>, ApiError> {
    sqlx::query_scalar!(
        r#"select role from tenant_members
           where tenant_id = $1 and user_id = $2 and left_at is null
           for no key update"#,
        tenant.get(),
        user_id
    )
    .fetch_optional(conn)
    .await?
    .map(|role| Role::from_db(&role))
    .transpose()
}

/// Deixa o membro inativo (`by`: quem removeu; `None`, saiu sozinho). Quem
/// chama já tirou as inscrições abertas e revogou os convites dele.
pub async fn deactivate(
    conn: &mut PgConnection,
    tenant: TenantId,
    user_id: i64,
    by: Option<i64>,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"update tenant_members set left_at = now(), removed_by = $3
           where tenant_id = $1 and user_id = $2 and left_at is null"#,
        tenant.get(),
        user_id,
        by
    )
    .execute(conn)
    .await?;
    Ok(())
}

pub async fn set_role(
    conn: &mut PgConnection,
    tenant: TenantId,
    user_id: i64,
    role: Role,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"update tenant_members set role = $3
           where tenant_id = $1 and user_id = $2 and left_at is null"#,
        tenant.get(),
        user_id,
        role.as_str()
    )
    .execute(conn)
    .await?;
    Ok(())
}

pub async fn set_nickname(
    conn: &mut PgConnection,
    tenant: TenantId,
    user_id: i64,
    nickname: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "update tenant_members set nickname = $3 where tenant_id = $1 and user_id = $2",
        tenant.get(),
        user_id,
        nickname
    )
    .execute(conn)
    .await?;
    Ok(())
}

/// Comunidades em que `user_id` é membro ativo, para a exclusão da conta (o
/// agendador age pelo sistema).
pub async fn active_tenants(
    conn: &mut PgConnection,
    user_id: i64,
) -> Result<Vec<TenantId>, sqlx::Error> {
    Ok(sqlx::query_scalar!(
        r#"select tenant_id from tenant_members
           where user_id = $1 and left_at is null
           order by tenant_id"#,
        user_id
    )
    .fetch_all(conn)
    .await?
    .into_iter()
    .map(TenantId::new)
    .collect())
}

/// Troca os apelidos de `user_id`, em toda comunidade em que ele já esteve,
/// pelo marcador de conta excluída (o vazio), e devolve essas comunidades.
pub async fn forget_nicknames(
    conn: &mut PgConnection,
    user_id: i64,
) -> Result<Vec<TenantId>, sqlx::Error> {
    Ok(sqlx::query_scalar!(
        r#"update tenant_members set nickname = '' where user_id = $1
           returning tenant_id"#,
        user_id
    )
    .fetch_all(conn)
    .await?
    .into_iter()
    .map(TenantId::new)
    .collect())
}
