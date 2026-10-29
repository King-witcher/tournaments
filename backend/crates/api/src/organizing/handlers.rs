use axum::{Json, extract::State};
use chrono::Utc;

use super::{db, types::Organizing};
use crate::{
    AppState,
    auth::CurrentUser,
    shared::{ApiError, ErrorBody},
};

#[utoipa::path(
    get,
    path = "/me/organizing",
    tag = "tournaments",
    responses(
        (status = OK, body = Organizing, description = "O que espera a organização em todas as comunidades em que quem vê é organizer ou owner ativo: mapas contestados a resolver (do mais antigo) e torneios na inscrição ou no check-in com o início marcado já passado (o critério do aviso `tournament.start_due`), com quantos fizeram check-in. Sem comunidade que organize, as listas vêm vazias. Muda pelo canal da pessoa (`me`)"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
    )
)]
pub async fn organizing(
    State(state): State<AppState>,
    user: CurrentUser,
) -> Result<Json<Organizing>, ApiError> {
    let (contested, start_due) = tokio::try_join!(
        db::contested(&state.db, user.id),
        db::start_due(&state.db, user.id, Utc::now()),
    )?;
    Ok(Json(Organizing {
        contested,
        start_due,
    }))
}
