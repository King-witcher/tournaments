//! Pareamentos gravados do suíço (`stage_pairings`): montar as rodadas das
//! linhas que `stage_rows` lê e gravar o pareamento de uma rodada nova.
//! Apagar fica no `discard`.

use engine::{core::ParticipantId, formats::swiss::SwissRound};
use serde::Deserialize;
use sqlx::PgConnection;

use crate::{shared::ApiError, tenants::TenantId, tournaments::TournamentId};

/// Uma linha de `stage_pairings`, como vem do JSON de `stage_rows`.
#[derive(Deserialize, Debug)]
pub struct PairingRow {
    round: i16,
    side_a: i64,
    side_b: Option<i64>,
}

/// As rodadas, da 1 em diante, das linhas na ordem (rodada e número). 500 se
/// as rodadas gravadas não forem 1..n (o banco garante; só um erro nosso
/// chega aqui).
pub fn rounds(rows: Vec<PairingRow>) -> Result<Vec<SwissRound>, ApiError> {
    let mut rounds: Vec<SwissRound> = Vec::new();
    for row in rows {
        if usize::try_from(row.round).ok() != Some(rounds.len()) {
            if usize::try_from(row.round - 1).ok() != Some(rounds.len()) {
                tracing::error!(round = row.round, "rodada do suíço fora da sequência");
                return Err(ApiError::internal());
            }
            rounds.push(SwissRound {
                pairs: Vec::new(),
                bye: None,
            });
        }
        let round = rounds.last_mut().ok_or_else(ApiError::internal)?;
        let a = ParticipantId(row.side_a);
        match row.side_b {
            Some(b) => round.pairs.push([a, ParticipantId(b)]),
            None => round.bye = Some(a),
        }
    }
    Ok(rounds)
}

/// Grava a rodada `round` (de 1 em diante) da fase `stage`: os pares de 1 em
/// diante e o bye por último, como o `engine` numera as partidas.
pub async fn insert_round(
    conn: &mut PgConnection,
    (tenant, tournament, stage): (TenantId, TournamentId, i16),
    round: u16,
    pairing: &SwissRound,
) -> Result<(), ApiError> {
    let round = i16::try_from(round).map_err(|_| ApiError::internal())?;
    let rows = pairing
        .pairs
        .iter()
        .map(|[a, b]| (a.0, Some(b.0)))
        .chain(pairing.bye.map(|p| (p.0, None)));
    for (number, (a, b)) in rows.enumerate() {
        let number = i16::try_from(number + 1).map_err(|_| ApiError::internal())?;
        sqlx::query!(
            r#"insert into stage_pairings
                   (tenant_id, tournament_id, stage, round, number, side_a, side_b)
               values ($1, $2, $3, $4, $5, $6, $7)"#,
            tenant.get(),
            tournament.get(),
            stage,
            round,
            number,
            a,
            b
        )
        .execute(&mut *conn)
        .await?;
    }
    Ok(())
}
