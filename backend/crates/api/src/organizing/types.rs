use chrono::{DateTime, Utc};
use serde::Serialize;
use utoipa::ToSchema;

use crate::tournaments::{NameTranslation, TournamentStatus};

/// O que espera a organização nas comunidades em que quem vê é organizer ou
/// owner ativo. Para quem não organiza nenhuma, as listas vêm vazias.
#[derive(Serialize, ToSchema)]
pub struct Organizing {
    /// Mapas contestados esperando a organização, do que espera há mais
    /// tempo ao mais recente.
    pub contested: Vec<ContestedGame>,
    /// Torneios na inscrição ou no check-in com o início marcado já passado,
    /// do início mais antigo ao mais recente.
    pub start_due: Vec<StartDue>,
}

/// O torneio de um item: a comunidade, o endereço e o nome.
#[derive(Serialize, ToSchema)]
pub struct PendingTournament {
    pub tenant_slug: String,
    pub tenant_name: String,
    pub slug: String,
    /// No idioma original.
    pub name: String,
    /// Nome nas traduções que existem.
    pub translations: Vec<NameTranslation>,
}

/// Um mapa contestado: resolve-se na página da partida
/// (`/{tenant}/{torneio}/matches/{key}`).
#[derive(Serialize, ToSchema)]
pub struct ContestedGame {
    pub tournament: PendingTournament,
    /// A partida: `wb-1-3`, `lb-2-1`, `gf-1-1`.
    pub key: String,
    /// Eliminação simples (o código curto da partida não diz a chave).
    pub single: bool,
    /// Número do mapa na série.
    pub game: i16,
    /// Nome do mapa jogado; `null` em série antiga, sem detalhe por mapa.
    pub map: Option<String>,
    /// Apelido de quem contestou; `null` se a conta não existe mais.
    pub contested_by: Option<String>,
    pub contested_at: DateTime<Utc>,
}

/// Um torneio cujo início marcado chegou sem ele ser iniciado: resolve-se na
/// aba Organização (`/{tenant}/{torneio}/manage`), iniciando ou mudando a
/// data.
#[derive(Serialize, ToSchema)]
pub struct StartDue {
    pub tournament: PendingTournament,
    /// `registration` ou `check_in`.
    pub status: TournamentStatus,
    pub starts_at: DateTime<Utc>,
    /// Com check-in, só quem fez entra na chave; sem, todos os inscritos.
    pub has_check_in: bool,
    /// Inscritos.
    pub registered: i64,
    /// Inscritos com check-in feito.
    pub checked_in: i64,
}
