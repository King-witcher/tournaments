//! Chave e resultados, mapa a mapa: ver a chave e uma partida, enviar um
//! mapa, confirmar ou contestar (`report`; o prazo conta de quando o mapa
//! foi liberado, `release`; quem contesta e até quando, também depois de
//! confirmado, `contest`), o capitão adversário aceitar a contestação (`accept`;
//! a organização é avisada depois, se ninguém resolver, pelo agendador,
//! `contest_notice`), e o organizador lançar ou corrigir mapas e dar
//! W.O. na série (`manage`) ou anular um mapa para ser jogado de novo
//! (`replay`). A série decidida que muda (correção ou contestação tardia) é
//! conferida e gravada em `correction`.
//!
//! Cada mapa da série é enviado e confirmado sozinho (`games`); a série
//! fecha quando os mapas confirmados dão a maioria ou empatam a série par (regra
//! do `engine`, `series::games`), e só a série decidida entra na chave
//! (`result`). A chave
//! (a fase de qualquer formato: eliminação, round robin ou suíço) nunca é
//! gravada: sai do `replay` do `engine` com a configuração da fase, os times
//! pelo seed, as séries decididas, na ordem em que passaram a valer, e, no
//! suíço, os pareamentos de cada rodada, que ficam gravados (`stage`,
//! `rounds`). Gerar e descartar a chave fica no início e no desfazer do
//! `lifecycle` (`start`, `discard`). Depois de toda mudança, `settle` anula os
//! mapas que ficaram depois da decisão, fecha as séries decididas, dá os W.O.
//! automáticos de quem desistiu ou saiu da comunidade (`withdrawals`), grava
//! o pareamento da rodada seguinte do suíço (`rounds`) e encerra ou reabre o
//! torneio conforme a chave. A confirmação por tempo é aplicada ao ler e
//! agir (`due`), e o agendador do tempo real a aplica nos torneios que
//! ninguém leu (`confirm_all_due`). O que
//! muda na chave vai para o outbox pelo `settle` e pelo início (`changes`),
//! com os avisos de partida pronta, mapa enviado e contestado. A desistência,
//! que fica no `lifecycle`, chama `settle_withdrawals`. O lugar de quem vê na
//! chave e a próxima partida dele, para `/me/tournaments`, saem de `place`
//! (`mine`), e onde terminou nos encerrados, de vários torneios numa leitura
//! só, de `final_places`; o de outro inscrito, para o perfil do membro, de
//! `place_of`. O
//! veto de mapas e campeões de cada partida, entre ela ficar pronta e o envio
//! de cada mapa, fica em `veto` (o relógio dos passos, `VetoClock`, roda no
//! `realtime`). O domínio `screenshots` lê a partida por `slots`; com
//! screenshot obrigatório, o envio do jogador exige um screenshot anexado
//! (`evidence`). No round robin, `progress` diz quem está jogando e qual série
//! sugerir a cada time, sem travar nada. Quem do time pode agir, pela regra do
//! torneio, e quem marca "Pronto" ficam em `acting`, num lugar só.

mod accept;
mod acting;
mod actions;
mod announce;
mod changes;
mod contest;
mod contest_notice;
mod correction;
mod db;
mod detail;
mod due;
mod evidence;
mod games;
mod handlers;
mod history;
mod key;
mod manage;
mod mine;
mod outcome;
mod progress;
mod release;
mod replay;
mod report;
mod result;
mod rounds;
mod settle;
mod slots;
mod stage;
mod tasks;
mod types;
pub mod veto;
mod view;
mod withdrawals;

use utoipa_axum::{router::OpenApiRouter, routes};

use crate::AppState;

pub use contest_notice::notify_contested;
pub use due::{apply_due_confirmations, confirm_all_due};
pub use mine::{Place, final_places, place, place_from, place_of};
pub use result::Side;
pub use slots::{MatchGames, games as match_games, lock_games as lock_match_games};
pub use stage::{discard, start};
pub use types::NextMatch;
pub use veto::{VetoClock, VetoPass};
pub use withdrawals::settle as settle_withdrawals;

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(handlers::get_bracket))
        .routes(routes!(handlers::get_match))
        .routes(routes!(report::report))
        .routes(routes!(report::confirm))
        .routes(routes!(report::contest))
        .routes(routes!(accept::accept_contest))
        .routes(routes!(manage::put_game))
        .routes(routes!(manage::set_result))
        .routes(routes!(replay::replay))
        .routes(routes!(
            veto::handlers::veto_ready,
            veto::handlers::veto_unready
        ))
        .routes(routes!(veto::handlers::veto_begin))
        .routes(routes!(veto::handlers::veto_step))
        .routes(routes!(veto::handlers::veto_undo))
        .routes(routes!(veto::handlers::veto_undo_request))
        .routes(routes!(veto::handlers::veto_undo_answer))
}
