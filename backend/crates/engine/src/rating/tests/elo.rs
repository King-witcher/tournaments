use proptest::prelude::*;

use super::*;

#[test]
fn expected_score() {
    assert_eq!(Elo::expected(1500, 1500), 0.5);
    let favorite = Elo::expected(1700, 1500);
    assert!((favorite - 0.759_747).abs() < 1e-6, "{favorite}");
    assert!((favorite + Elo::expected(1500, 1700) - 1.0).abs() < 1e-12);
}

#[test]
fn k_decays_with_games() {
    let config = EloConfig::default();
    let k: Vec<f64> = [0, 1, 10, 50].map(|n| config.k(n)).to_vec();
    assert_eq!(k[0], 80.0);
    for (got, want) in k.iter().zip([80.0, 75.432_196, 49.658_213, 32.323_421]) {
        assert!((got - want).abs() < 1e-6, "{got} != {want}");
    }
    assert!((0..200).all(|n| config.k(n) > config.k(n + 1) && config.k(n) > 32.0));
    // K mínimo igual ao máximo: K fixo.
    let fixed = EloConfig {
        k_min: 24,
        k_max: 24,
        ..config
    };
    assert_eq!(fixed.k(0), 24.0);
    assert_eq!(fixed.k(500), 24.0);
}

#[test]
fn new_player_against_veteran_is_not_zero_sum() {
    // O novo (K 80) ganha muito; o veterano com 50 jogos (K 32,3) perde pouco.
    assert_eq!(elo().points(1500, 0, 1700, 50), (61, 25));
    // O veterano favorito ganha pouco; o novo perde mais do que isso.
    assert_eq!(elo().points(1700, 50, 1500, 0), (8, 19));
}

#[test]
fn same_k_is_zero_sum() {
    assert_eq!(elo().points(1600, 10, 1500, 10), (18, 18));
    assert_eq!(elo().points(1500, 10, 1600, 10), (32, 32));
}

#[test]
fn heavy_favorite_can_win_nothing() {
    // 0,18 ponto arredonda para 0.
    assert_eq!(elo().points(2400, 100, 1500, 100), (0, 0));
}

proptest! {
    #[test]
    fn equal_games_means_equal_points(
        winner in 100..3000i32,
        loser in 100..3000i32,
        games in 0..300u32,
    ) {
        let (gain, loss) = elo().points(winner, games, loser, games);
        prop_assert_eq!(gain, loss);
        prop_assert!(gain >= 0 && f64::from(gain) <= EloConfig::default().k(games).ceil());
    }

}

#[test]
fn custom_config() {
    let config = EloConfig {
        initial: 1200,
        k_min: 20,
        k_max: 40,
        k_decay: 5,
    };
    let replay = replay(&Elo::new(config).unwrap(), &[beats(A, B)]).unwrap();
    assert_eq!(ratings(&replay), [(A, 1220), (B, 1180)]);
}

#[test]
fn config_limits() {
    let default = EloConfig::default();
    assert_eq!(default.check(), Ok(()));
    let bad = [
        (
            EloConfig {
                initial: 99,
                ..default
            },
            EloConfigError::Initial {
                min: 100,
                max: 3000,
            },
        ),
        (
            EloConfig {
                k_min: 0,
                ..default
            },
            EloConfigError::KMin { min: 1, max: 100 },
        ),
        (
            EloConfig {
                k_max: 201,
                ..default
            },
            EloConfigError::KMax { min: 1, max: 200 },
        ),
        (
            EloConfig {
                k_decay: 0,
                ..default
            },
            EloConfigError::KDecay { min: 1, max: 100 },
        ),
        (
            EloConfig {
                k_min: 50,
                k_max: 40,
                ..default
            },
            EloConfigError::KOrder,
        ),
    ];
    for (config, error) in bad {
        assert_eq!(config.check(), Err(error), "{config:?}");
        assert_eq!(Elo::new(config), Err(error));
    }
}

#[test]
fn accepted_limits() {
    let default = EloConfig::default();
    for config in [
        EloConfig {
            initial: 100,
            k_min: 1,
            k_max: 1,
            k_decay: 1,
        },
        EloConfig {
            initial: 3000,
            k_min: 100,
            k_max: 200,
            k_decay: 100,
        },
        EloConfig {
            k_min: 40,
            k_max: 40,
            ..default
        },
    ] {
        assert_eq!(config.check(), Ok(()), "{config:?}");
    }
}

#[test]
fn exact_half_rounds_away_from_zero_on_both_sides() {
    // K fixo 25 e ratings iguais: 12,5 para cada lado vira 13, e a soma
    // continua zero.
    let fixed = Elo::new(EloConfig {
        k_min: 25,
        k_max: 25,
        ..EloConfig::default()
    })
    .unwrap();
    assert_eq!(fixed.points(1500, 3, 1500, 3), (13, 13));
}

#[test]
fn regression_values() {
    // Valores fixos com K longe do inteiro e placar esperado longe de 0,5:
    // uma libm que diferisse no último bit mudaria algum deles.
    let config = EloConfig::default();
    assert!((config.k(100) - 32.002_179_196_628_6).abs() < 1e-12);
    assert_eq!(config.k(1000), 32.0);
    assert_eq!(config.k(u32::MAX), 32.0);
    assert_eq!(elo().points(1834, 137, 1622, 3), (7, 15));
    assert_eq!(elo().points(1290, 1000, 2010, 25), (32, 35));
    // Quase o mesmo K (7 e 8 jogos) já não dá soma zero.
    assert_eq!(elo().points(1501, 7, 1499, 8), (28, 27));
}
