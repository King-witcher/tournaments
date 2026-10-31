# Decisões do projeto

Cada decisão com o motivo; o que o código mostra por si só fica de fora. Está dividido por área em `docs/decisions/`. Atualize o arquivo da área quando algo mudar, em poucas linhas: decisão, motivo e o número ou limite que importa. "Do usuário" é decisão dele; "assumido" é minha, fácil de mudar.

Vocabulário dos arquivos: **partida** = `match` (a série; na interface, "série") e **mapa** = `game` (um jogo; na interface, "partida"). Glossário da interface em `web/CLAUDE.md`.

| Arquivo | O que tem |
|---|---|
| `decisions/stack.md` | Produto (nome e logo), stack e hospedagem, Redis (cota e o que entra), ids e segredos, repositório e CI, organização do código, estratégia de testes |
| `decisions/auth.md` | Logins (Discord, Google, senha), PKCE, vincular, limites de tentativa, CSRF e cabeçalhos, sessão |
| `decisions/mail.md` | Resend, forma canônica e domínios recusados, cadastro por código/link, recuperação, limites de envio e por IP, avisos de segurança |
| `decisions/tenants.md` | Comunidades, papéis, convites, membros inativos, posse |
| `decisions/deletion.md` | Excluir conta (anonimiza) e excluir comunidade, ambas agendadas para 7 dias |
| `decisions/tournaments.md` | Criação e edição, inscrição aberta/só por link, data de início, ciclo e check-in, inscrição, seeding, i18n do conteúdo |
| `decisions/formats.md` | Eixos de formato, times em todos os modos, round robin e suíço, TDM (formação, janela de campeões, substituição) |
| `decisions/matches.md` | Envio mapa a mapa, regras da série, contestação e prazos, ações do organizador, desistência e W.O., mapas |
| `decisions/veto.md` | Modo do mapa, pool, cronômetro, desfazer, draft de campeões da série, não repetição, orçamento de campeões, replay |
| `decisions/ratings.md` | Elo por mapa, K que decai, times, recálculo derivado, ajustes, ranking |
| `decisions/screenshots.md` | Regras do screenshot do placar, bucket S3, upload assinado, exclusão sem órfãos |
| `decisions/realtime.md` | WebSocket, outbox, agendador, avisos, Web Push, aba escondida, sininho |
| `decisions/discord.md` | Webhook por comunidade, fila, menções, catálogo de avisos |
| `decisions/database.md` | O que o banco recusa por conta própria e o que fica de fora de propósito |
| `decisions/interface.md` | Carbon, tema, confirmação, foco, validação, home, "Seus torneios", pendências da organização |
| `decisions/tournaments-ui.md` | Página do torneio, tela da partida, formulário de torneio, aba Regras |

As migrations já aplicadas citam este arquivo por nome de seção e não podem ser editadas (o sqlx confere o checksum do arquivo inteiro, comentário incluído). Seção antiga → arquivo: Ids, Redis, Repositório, Testes → `stack`; Autenticação → `auth`; Email → `mail`; Multi-tenant → `tenants`; Torneios → `tournaments`; Formatos, Times, TDM → `formats`; Resultados → `matches` (e `screenshots`); Veto e campeões → `veto`; Rating → `ratings`; Consistência do banco → `database`; Excluir conta e comunidade → `deletion`; Notificações → `discord` e `realtime`; Tempo real e avisos → `realtime`.

## Em aberto

- Repetição de mapa por time ao longo do torneio, para as partidas transmitidas não mostrarem sempre os mesmos mapas (ideia do usuário em 30/10; pede brainstorming antes).
- Critérios de desempate de FFA por pontos e grupos; rating para FFA (Elo par a par?).
