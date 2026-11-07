//! Handlers das imagens das regras: o envio é o do logo e da capa (URL
//! assinada, PUT do navegador, registro conferido por HEAD), pedido num
//! torneio pela organização, mas a imagem é registrada na comunidade
//! (`tournaments::register_rules_image`), porque a cópia de um torneio leva o
//! texto das regras com as URLs. Registrada, ela só fica se um texto passar a
//! usá-la (`tournaments` confere ao gravar as regras); a nunca usada sai na
//! rotina diária, depois de um dia. O objeto guarda o cache de uma hora
//! (`object::Place::cache_control`).

use axum::{Json, extract::State, http::StatusCode};
use chrono::Utc;

use super::{
    db::{self, Scope},
    handlers::{image, storage, tournament},
    object::Place,
    types::{ImageUpload, NewImageUpload, PublicImage, TournamentImagePath, TournamentUploadPath},
    upload::{self, NOT_FOUND, Target},
};
use crate::{
    AppState,
    shared::{ApiError, ErrorBody, JsonBody, PathParams},
    tenants::{Organizer, TenantMember},
    tournaments::{self, StoredTournament},
};

/// Imagens das regras registradas e ainda sem texto, por comunidade: o upload
/// registrado não conta no limite por hora (`upload`), e sem isto daria para
/// encher o bucket até a rotina diária.
const MAX_UNUSED: i64 = 20;

fn rules<'a>(member: &'a TenantMember<Organizer>, t: &StoredTournament) -> Target<'a> {
    Target {
        scope: Scope::Rules {
            tenant: member.tenant,
            tournament: t.id,
        },
        user: member.user.id,
        place: Place::Rules {
            tenant: &member.slug,
        },
    }
}

#[utoipa::path(
    post,
    path = "/tenants/{slug}/tournaments/{tournament}/rules/images/uploads",
    tag = "images",
    params(
        ("slug" = String, Path, description = "Slug do tenant"),
        ("tournament" = String, Path, description = "Slug do torneio"),
    ),
    request_body = NewImageUpload,
    responses(
        (status = CREATED, body = ImageUpload, description = "URL para o navegador mandar uma imagem das regras direto ao bucket (como a do logo, com o `Cache-Control` de uma hora); depois, registrar com `.../rules/images/uploads/{upload}/attach` e pôr a URL devolvida no Markdown das regras. Organizer e owner, em qualquer estado do torneio"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = FORBIDDEN, body = ErrorBody, description = "tenant.forbidden"),
        (status = NOT_FOUND, body = ErrorBody, description = "tenant.not_found, tournament.not_found"),
        (status = CONFLICT, body = ErrorBody, description = "tenant.deletion_scheduled"),
        (status = UNPROCESSABLE_ENTITY, body = ErrorBody, description = "images.size_invalid {max} (bytes), request.invalid_body (tipo fora de image/jpeg)"),
        (status = TOO_MANY_REQUESTS, body = ErrorBody, description = "images.too_many_uploads {max} (contando logos, capas e fotos), images.too_many_unused {max} (imagens das regras registradas na comunidade que nenhum texto usa ainda)"),
        (status = SERVICE_UNAVAILABLE, body = ErrorBody, description = "images.not_configured"),
    )
)]
pub async fn create_rules_image_upload(
    State(state): State<AppState>,
    member: TenantMember<Organizer>,
    PathParams(path): PathParams<TournamentImagePath>,
    JsonBody(body): JsonBody<NewImageUpload>,
) -> Result<(StatusCode, Json<ImageUpload>), ApiError> {
    let storage = storage(&state)?;
    let t = tournament(&state, &member, &path.tournament).await?;
    let mut conn = state.db.acquire().await?;
    if tournaments::unused_rules_images(&mut conn, member.tenant).await? >= MAX_UNUSED {
        return Err(
            ApiError::new(StatusCode::TOO_MANY_REQUESTS, "images.too_many_unused")
                .with_params([("max", MAX_UNUSED)]),
        );
    }
    drop(conn);
    let target = rules(&member, &t);
    let upload = upload::request(&state.db, storage, &target, body, Utc::now()).await?;
    Ok((StatusCode::CREATED, Json(upload)))
}

#[utoipa::path(
    post,
    path = "/tenants/{slug}/tournaments/{tournament}/rules/images/uploads/{upload}/attach",
    tag = "images",
    params(
        ("slug" = String, Path, description = "Slug do tenant"),
        ("tournament" = String, Path, description = "Slug do torneio"),
        ("upload" = i64, Path, description = "O `id` do upload"),
    ),
    responses(
        (status = OK, body = PublicImage, description = "Upload conferido no bucket e registrado como imagem das regras da comunidade: a URL vale nas regras (e nas traduções) de qualquer torneio dela. Sem texto que a use em um dia, sai do bucket"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = FORBIDDEN, body = ErrorBody, description = "tenant.forbidden"),
        (status = NOT_FOUND, body = ErrorBody, description = "tenant.not_found, tournament.not_found, images.not_found (upload de outra pessoa, de outro torneio, de um logo ou capa, já registrado ou vencido)"),
        (status = CONFLICT, body = ErrorBody, description = "images.upload_missing, tenant.deletion_scheduled"),
        (status = UNPROCESSABLE_ENTITY, body = ErrorBody, description = "images.upload_invalid (o upload é descartado)"),
        (status = SERVICE_UNAVAILABLE, body = ErrorBody, description = "images.not_configured"),
    )
)]
pub async fn attach_rules_image(
    State(state): State<AppState>,
    member: TenantMember<Organizer>,
    PathParams(path): PathParams<TournamentUploadPath>,
) -> Result<Json<PublicImage>, ApiError> {
    let storage = storage(&state)?;
    let t = tournament(&state, &member, &path.tournament).await?;
    let pending = upload::verify(&state.db, storage, &rules(&member, &t), path.upload).await?;
    let mut tx = state.db.begin().await?;
    // A imagem primeiro: o upload que sai já é dela e não vai para a fila.
    tournaments::register_rules_image(&mut tx, member.tenant, &pending.object_key).await?;
    if !db::delete_upload(&mut tx, pending.id).await? {
        return Err(NOT_FOUND);
    }
    tx.commit().await?;
    image(&pending)
}
