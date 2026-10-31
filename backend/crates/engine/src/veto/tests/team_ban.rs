//! A proteção do ban nos times: quem bane precisa que sobre campeão para
//! cada jogador do elenco em todas as partidas, contado jogador a jogador,
//! com o mesmo pior caso do orçamento (`champions::required_pool`).

use super::*;
use crate::champions::NoRepeatWho;

/// 2v2 com `bans`, B no seed melhor: A bane primeiro.
fn a_bans_first(rules: VetoRules, best_of: u8) -> Veto {
    let mut v = begun(rules, best_of, B);
    v.set_rosters(open(&[P1, P2], &[P3, P4])).unwrap();
    assert_eq!(acting(&v), (Action::Ban, Target::Champion, vec![A]));
    v
}

/// 2v2 md1, 1 ban de cada lado, pela regra do jogador.
fn by_player() -> Veto {
    a_bans_first(with_rule(team_rules(1, 2, false), NoRepeatWho::Player), 1)
}

fn used(pairs: &[(PlayerId, i64)]) -> Vec<(PlayerId, ChampionId)> {
    pairs
        .iter()
        .map(|(p, c)| (*p, ChampionId(100 + c)))
        .collect()
}

fn used_by<'a>(champions: &'a [ChampionId], used_by: &'a [(PlayerId, ChampionId)]) -> Context<'a> {
    Context {
        used_by,
        ..free(champions)
    }
}

#[test]
fn each_player_counts_only_what_he_used() {
    // Pool de 6. P1 usou o 1 e o 2 em outras séries; P2, o 3 e o 1. Cada um
    // tem 4 livres, o que basta: o ban de A, o de B e os dois de A na
    // partida. Contar o que o elenco todo usou (1, 2 e 3) deixaria só 3 e
    // prenderia o ban.
    let v = by_player();
    let champions = champions(6);
    let pairs = used(&[(P1, 1), (P1, 2), (P2, 3), (P2, 1)]);
    let ctx = used_by(&champions, &pairs);
    assert_eq!(v.options(A, &ctx), [1, 2, 3, 4, 5, 6].map(champ).to_vec());
}

#[test]
fn the_ban_spares_what_a_short_player_can_pick() {
    // Pool de 5, os mesmos usos: P1 tem 3, 4 e 5; P2, 2, 4 e 5. Os dois
    // ficariam sem campeão: A só bane o 1, que nenhum dos dois pode escolher.
    let v = by_player();
    let champions = champions(5);
    let pairs = used(&[(P1, 1), (P1, 2), (P2, 3), (P2, 1)]);
    let ctx = used_by(&champions, &pairs);
    assert_eq!(v.options(A, &ctx), [champ(1)]);
}

#[test]
fn with_nothing_safe_the_protection_gives_way() {
    // Pool de 5: P1 usou o 1 e o 2; P2, o 3 e o 4. Todo campeão serve a
    // algum dos dois: a proteção cede, como a regra, para o veto não travar.
    let v = by_player();
    let champions = champions(5);
    let pairs = used(&[(P1, 1), (P1, 2), (P2, 3), (P2, 4)]);
    let ctx = used_by(&champions, &pairs);
    assert_eq!(v.options(A, &ctx).len(), 5);
}

#[test]
fn the_team_rule_counts_the_whole_team() {
    // Pool de 6, pela regra do time: A já usou 4 fora da série e tem 101 e
    // 102 para os dois. B ainda bane uma vez: se A banisse um dos dois, B
    // tiraria o outro.
    let champions = champions(6);
    let used = [103, 104, 105, 106].map(ChampionId);
    let ctx = ctx(&champions, [&used, &[]]);
    let mut v = a_bans_first(with_rule(team_rules(1, 2, false), NoRepeatWho::Team), 1);
    assert_eq!(v.options(A, &ctx), [3, 4, 5, 6].map(champ));
    step_in(&mut v, A, champ(3), &ctx).unwrap();
}

#[test]
fn the_team_rule_repeating_in_the_game_needs_the_budget_worst_case() {
    // 2v2 md3, regra do time, colegas podem repetir na partida: no pior caso
    // do orçamento, as duas primeiras partidas gastam 2 cada e a última 1:
    // A precisa de 5 dos seus, além do ban dele e do de B. Com 6 livres
    // (pool de 7, 1 usado fora), banir um dos seus deixaria B levá-lo a 4.
    let champions = champions(7);
    let used = [ChampionId(107)];
    let ctx = ctx(&champions, [&used, &[]]);
    let v = a_bans_first(with_rule(team_rules(1, 2, true), NoRepeatWho::Team), 3);
    assert_eq!(v.options(A, &ctx), [champ(7)]);
    // Com 8, cabe.
    let eight = super::champions(8);
    assert_eq!(v.options(A, &super::ctx(&eight, [&used, &[]])).len(), 8);
}

#[test]
fn without_mirror_the_opponents_count_too() {
    // Pool de 6, sem regra e sem espelho: A precisa de 2 para o time e de
    // mais 2 que B pode pegar na partida, além do próprio ban e do de B.
    let mut v = a_bans_first(no_mirror(team_rules(1, 2, false)), 1);
    let champions = champions(6);
    // 1 + 1 + 2 + 2 = 6: cabe, por pouco.
    assert_eq!(v.options(A, &free(&champions)).len(), 6);
    // Com 5, todo campeão é de A: cede.
    let five = champions[..5].to_vec();
    assert_eq!(v.options(A, &free(&five)).len(), 5);
    step_in(&mut v, A, champ(1), &free(&five)).unwrap();
}
