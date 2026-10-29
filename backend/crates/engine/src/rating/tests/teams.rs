use proptest::prelude::*;

use super::*;

fn adjust(player: PlayerId, delta: i32) -> Event {
    Event::Adjustment { player, delta }
}

#[test]
fn equal_teams_of_new_players_move_like_a_duel() {
    let replay = replay_of(&[team_beats(&[A, B], &[C, D])]);
    assert_eq!(
        ratings(&replay),
        [(A, 1540), (B, 1540), (C, 1460), (D, 1460)]
    );
    let change = |delta, after| Change { delta, after };
    assert_eq!(
        replay.effects,
        [Effect::Game {
            winners: vec![change(40, 1540), change(40, 1540)],
            losers: vec![change(-40, 1460), change(-40, 1460)],
        }]
    );
    let a = replay.standings[&A];
    assert_eq!((a.games, a.wins, a.losses), (1, 1, 0));
}

#[test]
fn the_expected_score_uses_the_mean_of_each_lineup() {
    // A 1600 e B 1400 (média 1500) contra C e D com 1500: placar esperado
    // 0,5, como entre dois times de 1500; os ajustes não mudam o K.
    let events = [
        adjust(A, 100),
        adjust(B, -100),
        team_beats(&[A, B], &[C, D]),
    ];
    let replay = replay_of(&events);
    assert_eq!(
        ratings(&replay),
        [(A, 1640), (B, 1440), (C, 1460), (D, 1460)]
    );
}

#[test]
fn each_player_uses_their_own_k() {
    // A já tem 50 jogos (K ~32,3); B é novo (K 80). Contra um time igual,
    // 1 - E = 0,5: A ganha 16 e B ganha 40.
    let mut events: Vec<Event> = (0..50)
        .map(|i| if i % 2 == 0 { beats(A, C) } else { beats(C, A) })
        .collect();
    // Deixa A e C de novo em 1500, para a conta ficar só no K.
    let after = replay_of(&events);
    for p in [A, C] {
        let delta = 1500 - after.standings[&p].rating;
        if delta != 0 {
            events.push(adjust(p, delta));
        }
    }
    events.push(team_beats(&[A, B], &[C, D]));
    let replay = replay_of(&events);
    let Some(Effect::Game { winners, losers }) = replay.effects.last() else {
        panic!("o último evento é um jogo");
    };
    let k = EloConfig::default();
    let half = |games| (k.k(games) * 0.5).round() as i32;
    assert_eq!(
        winners.iter().map(|c| c.delta).collect::<Vec<_>>(),
        [half(50), half(0)]
    );
    assert_eq!(
        losers.iter().map(|c| c.delta).collect::<Vec<_>>(),
        [-half(50), -half(0)]
    );
}

#[test]
fn sides_of_different_sizes() {
    // Um contra dois: médias 1500 e 1500, cada um com o próprio K.
    let replay = replay_of(&[team_beats(&[A], &[B, C])]);
    assert_eq!(ratings(&replay), [(A, 1540), (B, 1460), (C, 1460)]);
}

#[test]
fn a_duel_through_the_team_api_is_the_duel() {
    assert_eq!(
        elo().team_points(&[(1834, 137)], &[(1622, 3)]),
        (vec![7], vec![15])
    );
}

#[test]
fn invalid_lineups_are_refused() {
    let cases = [
        (team_beats(&[], &[B]), RatingError::EmptySide { index: 1 }),
        (team_beats(&[A], &[]), RatingError::EmptySide { index: 1 }),
        (
            team_beats(&[A, B], &[C, A]),
            RatingError::RepeatedPlayer {
                index: 1,
                player: A,
            },
        ),
        (
            team_beats(&[A, A], &[C]),
            RatingError::RepeatedPlayer {
                index: 1,
                player: A,
            },
        ),
    ];
    for (event, error) in cases {
        let events = [beats(C, D), event];
        assert_eq!(replay(&elo(), &events), Err(error), "{events:?}");
    }
}

proptest! {
    #[test]
    fn one_player_per_side_is_the_duel(
        winner in 100..3000i32,
        loser in 100..3000i32,
        winner_games in 0..300u32,
        loser_games in 0..300u32,
    ) {
        let (gains, losses) =
            elo().team_points(&[(winner, winner_games)], &[(loser, loser_games)]);
        let (gain, loss) = elo().points(winner, winner_games, loser, loser_games);
        prop_assert_eq!((gains, losses), (vec![gain], vec![loss]));
    }

    #[test]
    fn same_games_in_a_game_is_zero_sum_per_player(
        ratings in prop::collection::vec(100..3000i32, 4),
        games in 0..300u32,
    ) {
        let side = |r: &[i32]| r.iter().map(|&x| (x, games)).collect::<Vec<_>>();
        let (gains, losses) = elo().team_points(&side(&ratings[..2]), &side(&ratings[2..]));
        prop_assert!(gains.iter().chain(&losses).all(|&p| p == gains[0]));
    }
}
