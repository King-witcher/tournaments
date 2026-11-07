use super::*;
use crate::champions::{NoRepeatRule, NoRepeatWho};
use crate::formats::{
    StageFormat,
    elimination::{EliminationConfig, EliminationKind},
    placement::PlacementConfig,
    round_robin::RoundRobinConfig,
    swiss::SwissConfig,
    table::{FinalTie, Tiebreakers},
};
use crate::series::SeriesLength;

fn double(default: u8, finals: u8) -> StageFormat {
    StageFormat::Elimination(EliminationConfig {
        kind: EliminationKind::Double {
            grand_final_reset: true,
        },
        series: SeriesLength { default, finals },
    })
}

fn single(default: u8, finals: u8) -> StageFormat {
    StageFormat::Elimination(EliminationConfig {
        kind: EliminationKind::Single,
        series: SeriesLength { default, finals },
    })
}

fn round_robin(best_of: u8) -> StageFormat {
    StageFormat::RoundRobin(RoundRobinConfig {
        best_of,
        tiebreakers: Tiebreakers::ROUND_ROBIN,
        final_tie: FinalTie::Seed,
    })
}

fn swiss(best_of: u8) -> StageFormat {
    StageFormat::Swiss(SwissConfig {
        best_of,
        rounds: None,
        tiebreakers: Tiebreakers::SWISS,
        final_tie: FinalTie::Seed,
    })
}

fn placement(best_of: u8) -> StageFormat {
    StageFormat::Placement(PlacementConfig { best_of })
}

fn rules(bans: u32, scope: Option<NoRepeatScope>) -> ChampionRules {
    ChampionRules {
        bans_per_series: bans,
        mirror: true,
        no_repeat: scope.map(|scope| NoRepeatRule {
            who: NoRepeatWho::Player,
            scope,
        }),
        same_team: false,
    }
}

fn of(formats: &[StageFormat]) -> Vec<StageSetup> {
    formats.iter().copied().map(StageSetup::of).collect()
}

fn setup(
    n: usize,
    champions: ChampionRules,
    pool: u32,
    stages: &[StageFormat],
) -> Result<(), EngineError> {
    setup_of(n, champions, pool, &of(stages))
}

fn setup_of(
    n: usize,
    champions: ChampionRules,
    pool: u32,
    stages: &[StageSetup],
) -> Result<(), EngineError> {
    TournamentSetup {
        team_size: 1,
        slots: n,
        champions,
        champion_pool: pool,
        stages,
    }
    .validate()
}

fn exhausted(required: u32, available: u32) -> Result<(), EngineError> {
    Err(EngineError::ChampionPoolExhausted {
        required,
        available,
    })
}

/// O erro `error` na fase 1.
fn first(error: EngineError) -> Result<(), EngineError> {
    Err(error.in_stage(1))
}

#[test]
fn no_stages() {
    assert_eq!(
        setup(8, rules(0, None), 17, &[]),
        Err(EngineError::NoStages)
    );
}

#[test]
fn invalid_best_of() {
    assert_eq!(
        setup(8, rules(0, None), 17, &[double(4, 5)]),
        first(EngineError::InvalidBestOf(4))
    );
}

#[test]
fn slots_are_validated() {
    let s = [double(3, 5)];
    assert_eq!(
        setup(1, rules(0, None), 17, &s),
        first(EngineError::NotEnoughParticipants { min: 2, got: 1 })
    );
    assert_eq!(
        setup(1025, rules(0, None), 17, &s),
        first(EngineError::TooManyParticipants {
            max: 1024,
            got: 1025
        })
    );
}

#[test]
fn team_size_and_bans_have_limits() {
    let s = [double(3, 5)];
    for size in [0, 5] {
        assert_eq!(
            teams(size, 40, rules(0, None), 17, &s),
            Err(EngineError::InvalidTeamSize { max: 4, got: size })
        );
    }
    assert_eq!(
        setup(8, rules(4, None), 17, &s),
        Err(EngineError::TooManyBans { max: 3, got: 4 })
    );
    assert_eq!(setup(8, rules(3, None), 17, &s), Ok(()));
    // Números absurdos não estouram a conta do orçamento.
    let huge = ChampionRules {
        bans_per_series: 3,
        ..rules(0, Some(NoRepeatScope::Tournament))
    };
    let budget = ChampionBudget {
        pool_size: 17,
        bans_per_series: u32::MAX,
        team_size: u32::MAX,
        mirror: false,
        same_team: false,
        games_in_scope: u32::MAX,
    };
    assert_eq!(champions::required_pool(huge.no_repeat, budget), u32::MAX);
}

#[test]
fn without_rule_the_series_needs_both_bans_plus_one() {
    let s = [double(3, 5)];
    assert_eq!(setup(8, rules(3, None), 6, &s), exhausted(7, 6));
    assert_eq!(setup(8, rules(1, None), 17, &s), Ok(()));
}

#[test]
fn tournament_scope_uses_worst_case_games() {
    let s = [double(3, 5)];
    let r = rules(1, Some(NoRepeatScope::Tournament));
    assert_eq!(setup(16, r, 17, &s), exhausted(33, 17));
}

#[test]
fn series_scope_uses_longest_series() {
    let s = [double(3, 5)];
    let r = rules(1, Some(NoRepeatScope::Series));
    assert_eq!(setup(16, r, 17, &s), Ok(()));
    assert_eq!(setup(16, r, 5, &s), exhausted(7, 5));
}

#[test]
fn teams_need_a_champion_per_player() {
    // 4v4, 3 bans, sem regra: 6 bans e 4 escolhas do time na partida.
    let s = of(&[double(3, 5)]);
    let tdm = |champions, pool| {
        TournamentSetup {
            team_size: 4,
            slots: 16,
            champions,
            champion_pool: pool,
            stages: &s,
        }
        .validate()
    };
    assert_eq!(tdm(rules(3, None), 9), exhausted(10, 9));
    let same = ChampionRules {
        same_team: true,
        ..rules(3, None)
    };
    assert_eq!(tdm(same, 9), Ok(()));
    // Sem espelho, também as 4 do adversário na partida.
    let no_mirror = ChampionRules {
        mirror: false,
        ..rules(3, None)
    };
    assert_eq!(tdm(no_mirror, 13), exhausted(14, 13));
    // Na série (md5): 6 bans, 4 partidas dele e 4 do time na última.
    assert_eq!(tdm(rules(3, Some(NoRepeatScope::Series)), 17), Ok(()));
    assert_eq!(
        tdm(rules(3, Some(NoRepeatScope::Series)), 13),
        exhausted(14, 13)
    );
}

#[test]
fn stage_and_tournament_scopes_take_the_worst_stage() {
    // As fases são isoladas: nos dois escopos vale a fase de maior pior caso,
    // não a soma.
    let a = swiss(3);
    let b = double(3, 5);
    let s = [a, b];
    let wa = a.worst_case_games(16).unwrap();
    let wb = b.worst_case_games(16).unwrap();
    assert!(wa > 0 && wb > wa);
    for scope in [NoRepeatScope::Stage, NoRepeatScope::Tournament] {
        let r = rules(0, Some(scope));
        assert_eq!(setup(16, r, wb, &s), Ok(()));
        assert_eq!(setup(16, r, wb - 1, &s), exhausted(wb, wb - 1));
    }
}

#[test]
fn table_formats_use_their_worst_case_and_limits() {
    let rr = [round_robin(3)];
    let sw = [swiss(3)];
    let r = rules(0, Some(NoRepeatScope::Tournament));
    // Round robin com 8: 7 séries md3; suíço com 16: 4 rodadas md3.
    assert_eq!(setup(8, r, 21, &rr), Ok(()));
    assert_eq!(setup(8, r, 17, &rr), exhausted(21, 17));
    assert_eq!(setup(16, r, 12, &sw), Ok(()));
    assert_eq!(
        setup(33, rules(0, None), 17, &rr),
        first(EngineError::TooManyParticipants { max: 32, got: 33 })
    );
}

fn teams(
    size: u32,
    slots: usize,
    champions: ChampionRules,
    pool: u32,
    stages: &[StageFormat],
) -> Result<(), EngineError> {
    TournamentSetup {
        team_size: size,
        slots,
        champions,
        champion_pool: pool,
        stages: &of(stages),
    }
    .validate()
}

#[test]
fn slots_below_two_teams_are_counted_in_players() {
    let s = [double(3, 5)];
    // 3 vagas com time de 4: nem um time; 7, um só. O mínimo vem em
    // jogadores, não em lados.
    for slots in [3, 7] {
        assert_eq!(
            teams(4, slots, rules(0, None), 17, &s),
            Err(EngineError::NotEnoughPlayers { min: 8, got: slots })
        );
    }
    assert_eq!(teams(4, 8, rules(0, None), 17, &s), Ok(()));
    // No duelo, continua o erro do formato.
    assert_eq!(
        teams(1, 1, rules(0, None), 17, &s),
        first(EngineError::NotEnoughParticipants { min: 2, got: 1 })
    );
}

#[test]
fn team_rule_with_duplicates_in_md5_by_stage_and_by_tournament() {
    // 3v3, 1 ban, regra do time podendo repetir no mapa, séries md5, 4
    // times: a cada mapa anterior o time gasta 3; no último, basta 1. Com
    // duas fases, vale a de maior pior caso (a dupla).
    let a = placement(5);
    let b = double(5, 5);
    let s = [a, b];
    let wa = a.worst_case_games(4).unwrap();
    let wb = b.worst_case_games(4).unwrap();
    assert!(wb > wa);
    let rule = |scope| ChampionRules {
        bans_per_series: 1,
        mirror: true,
        no_repeat: Some(NoRepeatRule {
            who: NoRepeatWho::Team,
            scope,
        }),
        same_team: true,
    };
    let required = |games: u32| 2 + (games - 1) * 3 + 1;
    for scope in [NoRepeatScope::Stage, NoRepeatScope::Tournament] {
        let need = required(wb);
        assert_eq!(teams(3, 12, rule(scope), need, &s), Ok(()));
        assert_eq!(
            teams(3, 12, rule(scope), need - 1, &s),
            exhausted(need, need - 1)
        );
    }
}

#[test]
fn sixteen_teams() {
    // 4v4 com 64 vagas, ou 67 (os 3 a mais não formam time): 16 times,
    // o mesmo pior caso de 16 no duelo. Regra do jogador no torneio, 1
    // ban: 2 bans, um por mapa anterior e 4 no último.
    let s = [double(3, 5)];
    let w = s[0].worst_case_games(16).unwrap();
    let r = rules(1, Some(NoRepeatScope::Tournament));
    let need = 2 + (w - 1) + 4;
    for slots in [64, 67] {
        assert_eq!(teams(4, slots, r, need, &s), Ok(()));
        assert_eq!(teams(4, slots, r, need - 1, &s), exhausted(need, need - 1));
    }
    // 68 vagas: 17 times, a chave dobra.
    assert_ne!(teams(4, 68, r, need, &s), Ok(()));
}

#[test]
fn elimination_only_as_the_last_stage() {
    for elimination in [single(3, 5), double(3, 5)] {
        assert_eq!(
            setup(8, rules(0, None), 17, &[elimination, placement(3)]),
            Err(EngineError::EliminationNotLast { stage: 1 })
        );
        assert_eq!(
            setup(8, rules(0, None), 17, &[round_robin(2), elimination]),
            Ok(())
        );
    }
    assert_eq!(
        setup(
            8,
            rules(0, None),
            17,
            &[round_robin(2), double(3, 5), placement(3)]
        ),
        Err(EngineError::EliminationNotLast { stage: 2 })
    );
}

#[test]
fn the_cut_and_check_in_start_at_the_second_stage() {
    let with = |advance, check_in| StageSetup {
        advance,
        check_in,
        ..StageSetup::of(placement(3))
    };
    let first_rules = |advance, check_in| {
        setup_of(
            8,
            rules(0, None),
            17,
            &[with(advance, check_in), StageSetup::of(placement(3))],
        )
    };
    let refused = Err(EngineError::EntryRulesFirst { stage: 1 });
    assert_eq!(first_rules(Some(4), false), refused);
    assert_eq!(first_rules(None, true), refused);
    assert_eq!(first_rules(None, false), Ok(()));

    let second = |advance| {
        setup_of(
            8,
            rules(0, None),
            17,
            &[StageSetup::of(round_robin(2)), with(advance, true)],
        )
    };
    let invalid = Err(EngineError::InvalidAdvance {
        stage: 2,
        min: 2,
        max: 128,
    });
    assert_eq!(second(Some(1)), invalid);
    assert_eq!(second(Some(129)), invalid);
    // Mais que os lados das vagas vale: entram todos.
    assert_eq!(second(Some(2)), Ok(()));
    assert_eq!(second(Some(128)), Ok(()));
}

#[test]
fn the_cut_sets_the_next_stage_size() {
    // Suíço de 64 → round robin com os 8 melhores: o teto do round robin (32)
    // vale para o corte, não para as vagas.
    let rr = |advance| StageSetup {
        advance,
        check_in: false,
        ..StageSetup::of(round_robin(3))
    };
    let stages = |advance| [StageSetup::of(swiss(3)), rr(advance)];
    assert_eq!(setup_of(64, rules(0, None), 17, &stages(Some(8))), Ok(()));
    assert_eq!(
        setup_of(64, rules(0, None), 17, &stages(None)),
        Err(EngineError::TooManyParticipants { max: 32, got: 64 }.in_stage(2))
    );
    // O orçamento usa o corte: round robin de 8 em md3, 21 partidas.
    let r = rules(0, Some(NoRepeatScope::Stage));
    let swiss_worst = swiss(3).worst_case_games(64).unwrap();
    let need = swiss_worst.max(21);
    assert_eq!(setup_of(64, r, need, &stages(Some(8))), Ok(()));
    assert_eq!(
        setup_of(64, r, need - 1, &stages(Some(8))),
        exhausted(need, need - 1)
    );
}

#[test]
fn a_stage_starts_with_its_real_sides() {
    let stages = [StageSetup::of(round_robin(2)), StageSetup::of(placement(3))];
    let t = |pool| TournamentSetup {
        team_size: 1,
        slots: 32,
        champions: rules(0, Some(NoRepeatScope::Stage)),
        champion_pool: pool,
        stages: &stages,
    };
    // A disputa de posições pede só a série dela, com qualquer número de
    // lados; o round robin de 32 em md2 (62 partidas) não conta.
    assert_eq!(t(3).validate_stage(1, 8), Ok(()));
    assert_eq!(t(2).validate_stage(1, 8), exhausted(3, 2));
    assert_eq!(
        t(17).validate_stage(1, 1),
        Err(EngineError::NotEnoughParticipants { min: 2, got: 1 }.in_stage(2))
    );
}
