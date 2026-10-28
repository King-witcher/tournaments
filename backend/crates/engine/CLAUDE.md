# engine (backend/crates/engine)

Regras de torneio, puras: sem axum, sqlx, tokio, I/O nem relógio; aleatoriedade por seed/RNG passado como parâmetro. Para implementar ou mudar um formato, use o subagente `format-engine`.

```
src/
  core.rs         tipos de mais de um formato: ParticipantId, MapId, MatchId, Match, Occupant,
                  Outcome, EngineError
  series/         melhor de N: mod.rs (tamanho e placar, validate_score), games.rs (a série
                  mapa a mapa: próximo mapa, troca, onde a série fecha e o que fica depois dela)
  seeding.rs      sorteio reproduzível pela seed (random_order, ChaCha8), tamanho e ordem da chave
  champions.rs    ChampionRules e o orçamento da regra de não repetição
  tournament.rs   TournamentSetup::validate: vagas, fases e campeões antes de gravar
  formats/
    mod.rs        StageFormat (o JSON gravado em tournament_stages.config)
    elimination/  mod.rs (config, estado, replay), build.rs (topologia), resolve.rs (estados), tests/
```

- O JSON de `StageFormat` e `ChampionRules` fica gravado no banco: mudar o serde deles exige migration dos dados. Os testes em `formats/tests.rs` fixam o formato.
- Um formato por pasta em `formats/` (`round_robin/`, `swiss/`, `ffa/`): `mod.rs` com a API pública (config, estado, `replay`) e um arquivo por etapa.
- O que não é formato fica no topo, como `scoring/` (pontos e desempate de FFA), `veto/` e `rating/`. Um arquivo até ~300 linhas, depois pasta.
- Tipo usado por mais de um formato vai para `core.rs`.
