# engine (backend/crates/engine)

Regras de torneio, puras: sem axum, sqlx, tokio, I/O nem relógio; aleatoriedade por seed/RNG passado como parâmetro. Para implementar ou mudar um formato, use o subagente `format-engine`.

Em `src/`: `core.rs` (tipos de mais de um formato: `ParticipantId`, `MatchId`, `Outcome`, `StageReport`, `EngineError`), `series/` (melhor de N, mapa a mapa), `seeding.rs`, `teams.rs` (formação do TDM), `champions.rs` (`ChampionRules`, orçamento do pool), `tournament.rs` (`TournamentSetup::validate`), `stages.rs` (fases ligadas: `StageSetup` com corte e check-in, quem entra na fase seguinte e a classificação final do torneio em fases), `rating/`, `veto/` (veto de mapas e draft de campeões) e `formats/` (um formato por pasta; `formats/stage.rs` expõe a fase de qualquer formato, que é o que a API usa). Cada `mod.rs` tem o `//!` com o papel dos arquivos.

- `StageFormat` e `ChampionRules` ficam em colunas do banco, e a tradução é da API (`tournaments/db/columns.rs`): campo novo neles exige coluna e migration. Mudar o comportamento de uma regra ou de um formato muda torneios já gravados, que saem do `replay`: `tests/replay_golden` congela isso; não atualize o esperado sem decidir o que acontece com eles.
- Um formato por pasta em `formats/`: `mod.rs` com a API pública (config, estado, `replay`) e um arquivo por etapa. O que mais de um formato usa e não é tipo fica numa pasta própria (`table/`, de round robin e suíço).
- O que não é formato fica no topo (`veto/`, `rating/`, futuramente `scoring/` para pontos e desempate de FFA). Arquivo até ~300 linhas, depois pasta. Tipo usado por mais de um formato vai para `core.rs`.
