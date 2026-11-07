//! O torneio em fases ligadas, pelas regras do `engine` (`stages`): quem é
//! candidato a cada fase da 2 em diante (quem jogou a anterior, disponível se
//! não desistiu antes do início dela e, com check-in, confirmou presença),
//! quem entra (`entry`), a classificação final do torneio (`standings`) e a
//! regra da correção na fase anterior depois do início da seguinte
//! (`check_next`): passa se quem entraria e a ordem continuam os mesmos.

use chrono::{DateTime, Utc};
use engine::{
    core::{ParticipantId, Placement},
    formats::StageBracket,
    stages::{self as es, Candidate, Entry, StageSetup},
};

use super::stage::{Chain, Stage};
use crate::{
    shared::ApiError,
    teams::StoredTeam,
    tournaments::{StageProgress, StoredTournament},
};

/// A ordem da classificação final de `bracket`: os participantes pelas
/// colocações, uma para cada um (a fase de tabela e a disputa de posições
/// terminadas). `None` antes do fim, ou se sobrou alguém sem colocação.
pub fn ranking(bracket: &StageBracket) -> Option<Vec<ParticipantId>> {
    if !bracket.is_finished() {
        return None;
    }
    let mut placements = bracket.placements();
    if placements.len() != bracket.participants().len() {
        return None;
    }
    placements.sort_by_key(|p| p.rank);
    Some(placements.into_iter().map(|p| p.participant).collect())
}

/// A ordem de agora de `bracket`, para mostrar os candidatos antes do fim: a
/// classificação final, se já há; senão, a tabela de agora (nas fases de
/// tabela) ou a ordem dos seeds.
pub fn provisional(bracket: &StageBracket) -> Vec<ParticipantId> {
    if let Some(order) = ranking(bracket) {
        return order;
    }
    match bracket.table() {
        Some(rows) => rows.iter().map(|r| r.participant).collect(),
        None => bracket.participants().to_vec(),
    }
}

/// Os candidatos à fase `number` (`setup`), na ordem `ranking` da anterior,
/// dos times dela (`previous`): disponível quem não desistiu antes de
/// `started_at` (o início da fase; sem ele, agora) e, se a fase pede check-in,
/// confirmou presença.
pub fn candidates<'a>(
    previous: &'a [StoredTeam],
    ranking: &[ParticipantId],
    number: i16,
    setup: &StageSetup,
    started_at: Option<DateTime<Utc>>,
) -> Vec<(Candidate, Option<&'a StoredTeam>)> {
    ranking
        .iter()
        .map(|&p| {
            let team = previous.iter().find(|t| t.id == p.0);
            let available = team.is_some_and(|t| {
                !t.withdrawn_before(started_at) && (!setup.check_in || t.checked_in_for(number))
            });
            (
                Candidate {
                    participant: p,
                    available,
                },
                team,
            )
        })
        .collect()
}

/// Quem entra na fase `number`, dos times `previous` da anterior, na ordem
/// `ranking` dela, com a disponibilidade de `started_at` (`candidates`).
pub fn entry(
    previous: &[StoredTeam],
    ranking: &[ParticipantId],
    number: i16,
    setup: &StageSetup,
    started_at: Option<DateTime<Utc>>,
) -> Entry {
    let list: Vec<Candidate> = candidates(previous, ranking, number, setup, started_at)
        .into_iter()
        .map(|(c, _)| c)
        .collect();
    es::entry(&list, setup.advance)
}

/// Para mostrar antes do início da fase `number`: quem entraria agora. Com o
/// check-in da fase aberto, só quem confirmou presença; antes de abrir (ou sem
/// check-in), todos que não desistiram.
pub fn projection(
    previous: &[StoredTeam],
    ranking: &[ParticipantId],
    number: i16,
    setup: &StageSetup,
    progress: StageProgress,
) -> Entry {
    let counted = StageSetup {
        check_in: setup.check_in && progress.check_in_opened_at.is_some(),
        ..*setup
    };
    entry(previous, ranking, number, &counted, None)
}

/// A classificação final do torneio com as posições já conhecidas: os da fase
/// atual pela colocação dela, quando ela é a última; depois, de trás para a
/// frente, os cortados e os ausentes de cada fase (`engine::stages`). Com uma
/// fase só, as colocações dela.
pub fn standings(chain: &Chain) -> Vec<Placement> {
    let Some(current) = chain.stages.last() else {
        return Vec::new();
    };
    let mut entries = Vec::with_capacity(chain.stages.len().saturating_sub(1));
    for pair in chain.stages.windows(2) {
        let (previous, next) = (&pair[0], &pair[1]);
        let (setup, progress) = (&next.setup, next.progress);
        let order = ranking(&previous.bracket).unwrap_or_else(|| {
            previous
                .entrants
                .iter()
                .map(StoredTeam::participant)
                .collect()
        });
        // Quem entrou é o que está gravado; o resto se divide pela
        // disponibilidade no início da fase.
        let entered: Vec<ParticipantId> =
            next.entrants.iter().map(StoredTeam::participant).collect();
        let mut e = Entry {
            entrants: entered.clone(),
            ..Entry::default()
        };
        for (c, _) in candidates(
            &previous.entrants,
            &order,
            next.number,
            setup,
            progress.started_at,
        ) {
            if entered.contains(&c.participant) {
                continue;
            }
            if c.available {
                e.cut.push(c.participant);
            } else {
                e.absent.push(c.participant);
            }
        }
        entries.push(e);
    }
    let last = if chain.last {
        current.bracket.placements()
    } else {
        Vec::new()
    };
    es::final_standings(&last, &entries)
}

/// A fase `stage` foi mexida depois do início da seguinte: refaz quem entraria
/// nela com a mesma disponibilidade do início (as presenças gravadas e as
/// desistências de antes dele; quem entrou com seed conta como disponível,
/// mesmo com uma desistência gravada com hora anterior ao início por uma
/// transação mais longa) e confere com os seeds gravados. Se mudam quem
/// entra ou a ordem (ou a fase deixou de terminar), 409 `stage.next_started`
/// `{stage}` com a fase seguinte: o caminho é desfazer o início dela.
pub fn check_next(tournament: &StoredTournament, stage: &Stage) -> Result<(), ApiError> {
    if !stage.next_started {
        return Ok(());
    }
    let next = stage.number + 1;
    let refused =
        || ApiError::conflict("stage.next_started").with_params([("stage", i64::from(next))]);
    let (setup, progress) = tournament.stage(next).ok_or_else(ApiError::internal)?;
    let order = ranking(&stage.bracket).ok_or_else(refused)?;
    let list: Vec<Candidate> =
        candidates(&stage.entrants, &order, next, setup, progress.started_at)
            .into_iter()
            .map(|(c, team)| Candidate {
                available: c.available || team.is_some_and(|t| t.stage_seed(next).is_some()),
                ..c
            })
            .collect();
    let now = es::entry(&list, setup.advance);
    let recorded = crate::teams::stage_entrants(&stage.teams, next);
    let same = now.entrants.len() == recorded.len()
        && now.entrants.iter().zip(&recorded).all(|(p, t)| p.0 == t.id);
    if same { Ok(()) } else { Err(refused()) }
}

/// Um time candidato à fase da 2 em diante, para a tela dela.
pub struct NextCandidate {
    pub team: StoredTeam,
    /// A posição na classificação da fase anterior (a de agora, antes do fim).
    pub previous_rank: u16,
    /// O seed com que entraria: antes do início, quem entraria agora
    /// (`projection`); depois, o gravado. `None`: suplente ou fora.
    pub projected_seed: Option<u16>,
}

/// A fase `number` (2 em diante) vista antes e depois do início: os
/// candidatos e quem entra.
pub struct NextStage {
    /// A fase anterior terminou: a ordem dos candidatos é a final.
    pub previous_finished: bool,
    /// Os candidatos, na ordem da classificação da anterior; vazio antes de
    /// ela começar.
    pub candidates: Vec<NextCandidate>,
    /// Quem entra se a fase começar agora, com as presenças confirmadas de
    /// verdade, na ordem dos seeds; vazio antes do fim da anterior.
    pub entrants: Vec<ParticipantId>,
}

/// A fase `number` (2 em diante) de `tournament`: refaz a anterior (uma das
/// fases começadas) para os candidatos e quem entra. Antes do início do
/// torneio ou da fase anterior, sem candidatos.
pub async fn next_stage(
    conn: &mut sqlx::PgConnection,
    tournament: &StoredTournament,
    number: i16,
) -> Result<NextStage, ApiError> {
    let empty = NextStage {
        previous_finished: false,
        candidates: Vec::new(),
        entrants: Vec::new(),
    };
    let previous = number - 1;
    if !tournament.status.has_started() || !tournament.stage_started(previous) {
        return Ok(empty);
    }
    let (setup, progress) = tournament.stage(number).ok_or_else(ApiError::internal)?;
    let stage = super::stage::load(conn, tournament, previous).await?;
    let finished = ranking(&stage.bracket);
    let order = finished
        .clone()
        .unwrap_or_else(|| provisional(&stage.bracket));
    let started = tournament.stage_started(number);
    let projected = if started {
        crate::teams::stage_entrants(&stage.teams, number)
            .iter()
            .map(StoredTeam::participant)
            .collect()
    } else {
        projection(&stage.entrants, &order, number, setup, progress).entrants
    };
    let seed_of = |p: ParticipantId| {
        projected
            .iter()
            .position(|x| *x == p)
            .and_then(|i| u16::try_from(i + 1).ok())
    };
    let candidates = (1..)
        .zip(&order)
        .filter_map(|(rank, &p)| {
            let team = stage.entrants.iter().find(|t| t.id == p.0)?;
            Some(NextCandidate {
                team: team.clone(),
                previous_rank: rank,
                projected_seed: seed_of(p),
            })
        })
        .collect();
    let entrants = match finished {
        Some(order) => entry(&stage.entrants, &order, number, setup, None).entrants,
        None => Vec::new(),
    };
    Ok(NextStage {
        previous_finished: stage.finished_at.is_some() && stage.bracket.is_finished(),
        candidates,
        entrants,
    })
}
