# Roadmap

Ordem de trabalho combinada. Decisões e motivos ficam em `decisions.md`; aqui fica o que fazer e em que ordem.

## Feito

- Scaffold: workspace Rust (`engine`, `api`, `migrate`), Next 16 com next-intl (pt, es), Biome, Bun, banco local.
- `engine`: eliminação simples e dupla (`formats/elimination.rs`), seeding padrão, validação de melhor de N, checagem de pool de campeões (`champions.rs`). Revisado pelo `rules-reviewer`.
- `api`: login com Discord, sessão em cookie (só o hash no banco), `/me`, logout, extractor `CurrentUser`, `ApiError`.
- `web`: menu do usuário (entrar/sair), erros de login traduzidos, status da API.

## Padrões assumidos (configuráveis, confirmar com o usuário quando surgir)

- Melhor de N: um valor padrão + um para a decisiva (final na simples; grande final e reset na dupla). Ex.: md3 + md5.
- Grande final com reset: opção do organizador.
- Ban de campeão pelo capitão; pick por cada jogador. Timeout no veto: escolha aleatória, desfazível pelo organizador.
- Corrigir resultado = refazer o estado com `replay` e o histórico corrigido; correção que contradiz partida posterior é recusada.

## Próximo: duelo de ponta a ponta

Um incremento por vez, cada um com migration, rotas (OpenAPI), testes `sqlx::test`, tipos regenerados no front e tela mínima.

1. **Tenants**
   - Tabelas `tenants` (slug único, nome), `tenant_members` (tenant, usuário, papel: owner/organizer/player), `tenant_invites` (hash do token, expiração, criador).
   - Criar tenant (limite por usuário: valor em aberto, sugerir 3), listar os meus, gerar link de convite, aceitar convite.
   - Extractor de tenant que valida a participação e o papel e entrega um `TenantId` tipado (regra do `CLAUDE.md`).
   - Front: `/[locale]/[tenant]`, criar tenant, aceitar convite.
2. **Torneio de duelo**
   - `tournaments` (tenant, slug, idioma original, nome, descrição, regras, status: rascunho → inscrição → check-in → em andamento → encerrado), `tournament_translations` (opcional por idioma), `stages` (ordem, config do formato em JSON validado pelos tipos do `engine`).
   - Ao criar: validar o formato e, se houver regra de não repetição, rodar `champions::check_budget` com o `worst_case_games` do formato. Bloquear com o código de erro e os números.
3. **Inscrição, check-in e seeding**
   - `participants` (torneio, jogador do tenant, seed, check-in). Janela de check-in; quem não fizer sai antes da chave.
   - Seeding manual, aleatório (seed salva para reproduzir) ou por rating (quando existir).
4. **Chave e resultados**
   - Gerar a chave com `EliminationBracket`; persistir só config, participantes e `match_reports` (histórico); reconstruir por `replay`.
   - Envio de resultado por um lado, janela de contestação, organizador resolve contestação.
   - Front: visualização da chave (simples e dupla), página da partida.

## Depois

- Tempo real (WebSocket no Axum) para chave, check-in e partida.
- Veto de mapas e pick/ban de campeões ao vivo, com regra de não repetição aplicada.
- Prints do placar (S3 genérico, URL assinada, WebP, apagar 30 dias após o fim).
- Rating (Elo, configurável por tenant).
- Outros formatos: FFA por pontos/frags, 2v2/3v3 com formação de times, round robin, suíço.
- Deploy: Render (API), Vercel (web), Supabase (Postgres), CI no GitHub Actions com `cargo sqlx prepare` para build offline.
