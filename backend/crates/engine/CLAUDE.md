# engine (backend/crates/engine)

Regras de torneio, puras: sem axum, sqlx, tokio, I/O nem relógio; aleatoriedade por seed/RNG passado como parâmetro. Para implementar ou mudar um formato, use o subagente `format-engine`.

```
src/
  core.rs         tipos de mais de um formato: ParticipantId, MapId, MatchId, Match, Occupant,
                  Outcome, StageReport (resultado + SeriesFrags), EngineError
  series/         melhor de N: mod.rs (tamanho e placar, validate_score), games.rs (a série
                  mapa a mapa: próximo mapa, troca, onde a série fecha e o que fica depois dela)
  seeding.rs      sorteio reproduzível pela seed (random_order, ChaCha8), tamanho e ordem da chave
  teams.rs        formação dos times do TDM: reservas, sorteio aleatório e balanceado (zigue-zague)
  champions.rs    ChampionRules e o orçamento do pool (bans, regra de não repetição, times)
  tournament.rs   TournamentSetup::validate: vagas, fases e campeões antes de gravar
  rating/         mod.rs (RatingSystem, Event, replay: o histórico na ordem dada → rating final
                  e o efeito de cada evento), elo.rs (Elo com K que decai com os jogos), tests/
  veto/           veto de mapas e ban/pick de campeões de uma série: mod.rs (regras, Veto,
                  replay, vez, opções), sequence.rs (ordem dos passos), draw.rs (sorteios
                  pela seed), team.rs (a escolha simultânea de campeões nos times), tests/
  formats/
    mod.rs        StageFormat (o JSON gravado em tournament_stages.config)
    stage.rs      StageBracket: a fase de qualquer formato (o que a API usa) e StageHistory
    elimination/  mod.rs (config, estado, replay), build.rs (topologia), resolve.rs (estados), tests/
    table/        o que round robin e suíço dividem: mod.rs (TableRow, critérios, compute),
                  league.rs (partidas de lados fixos e a tabela), tiebreak.rs (ordem e
                  confronto direto), tests.rs
    round_robin/  mod.rs (config, estado, replay), schedule.rs (método do círculo), tests/
    swiss/        mod.rs (config, SwissRound, estado, replay, next_round), pairing.rs (bye,
                  revanches, preferência), matching.rs (emparelhamento máximo de Edmonds), tests/
```

- O JSON de `StageFormat` e `ChampionRules` fica gravado no banco: mudar o serde deles exige migration dos dados. Os testes em `formats/tests.rs` fixam o formato.
- Um formato por pasta em `formats/` (`round_robin/`, `swiss/`, `ffa/`): `mod.rs` com a API pública (config, estado, `replay`) e um arquivo por etapa. O que mais de um formato usa e não é tipo fica numa pasta própria em `formats/` (`table/`).
- O que não é formato fica no topo, como `scoring/` (pontos e desempate de FFA), `veto/` e `rating/`. Um arquivo até ~300 linhas, depois pasta.
- Tipo usado por mais de um formato vai para `core.rs`.
