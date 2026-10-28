# Roadmap

O que fazer e em que ordem. Regras e motivos ficam em `decisions.md`; o detalhe de cada incremento, no `git log`.

## Próximo

Um incremento por vez, cada um com migration, rotas (OpenAPI), testes `sqlx::test`, tipos regenerados no front e tela mínima.

1. **Tempo real** (WebSocket no Axum, `tokio::broadcast`): hoje chave, check-in, partida e "Seus torneios" só mudam ao recarregar. Um agendador chama `matches::apply_due_confirmations` (hoje aplicada ao ler e agir; já dá os W.O. automáticos) para os torneios com pendente vencido e avisa quem vê. Além de partida e check-in, avisar desistência e W.O. automático (`lifecycle::leave_tournaments` mexe em vários torneios de uma vez), membro que saiu ou foi removido (perde o acesso na hora) e posse transferida.
2. **Veto** de mapas e pick/ban de campeões ao vivo, com a regra de não repetição. Entra entre a partida ficar pronta e o envio: o pool por torneio sai de `maps`, e o envio de cada mapa passa a seguir o veto. Partida de quem desistiu não chega ao veto: vira W.O. ao ficar pronta.
3. **Prints do placar** (S3 genérico, URL assinada, WebP, apagar 30 dias após o fim).
4. **Rating** (Elo, configurável por tenant), por mapa a partir de `match_games` (série sem detalhe conta pelos mapas vencidos). W.O. não conta.

## Depois

- Outros formatos: FFA por pontos/frags, 2v2/3v3 com formação de times, round robin, suíço.
- CI no GitHub Actions conferindo que o `.sqlx/` está em dia (`SQLX_OFFLINE=true cargo check`).

## Pendências

O que ficou de fora de incrementos já feitos, com o que falta.

- **Sessões abertas na conta**: a API lista e encerra (`/auth/sessions`); falta a seção na conta (navegador, desde quando, "esta sessão", encerrar uma ou as outras).
- **Conferir a limpeza da `shared/ui` no navegador** (só passou por tipo, lint e build): `FormModal` de "inscrever alguém" (foco, Enter, foco de volta), `PassiveModal` do link de inscrição, anular mapa com nota, `FormActions` lado a lado, `RadioGroup` do mapa a 390px, listas vazias com a ação na barra, foco no campo recusado (apelido, comunidade).
- **Foco depois das ações da organização nas listas** (inscrever, tirar, check-in, desistência, seeding, convites e links): o item some ou muda de lugar, e o alvo certo é a lista.
- **Publicar com o check-in já aberto** leva direto ao check-in (hoje com aviso). A alternativa, publicar suspender a abertura automática, muda a API: decidir com o usuário.
- **Excluir conta e excluir comunidade**: regra para o usuário decidir ("Consistência do banco" no `decisions.md`).
- **Aviso fora do site** (Discord, email): precisa de bot ou provedor e de decidir o que vale aviso.
- **Tirar um W.O.**: série decidida por W.O. não reabre, então corrigir a partida anterior a ela só se resolve desfazendo o início. Criar a rota se aparecer na prática.
- **Pendências da organização em todas as comunidades**: "Seus torneios" só mostra as partidas de quem joga; contestações a resolver ficam na página de cada torneio.
- **Histórico completo de quem vê**: "Seus torneios" mostra só os 5 encerrados mais recentes; a lista inteira vai para uma página de perfil, com o rating.
- **Home de quem não entrou** mostra o esqueleto por um instante (ler cookies fica dentro do `<Suspense>` com Cache Components); tirar pede uma home estática para quem não tem cookie, decidida no proxy.
- **"Não encontrado" responde 200** (o conteúdo chega por streaming depois do status); corrigir pede `notFound()` fora do `Suspense`, o que mexe na estrutura das páginas.

## Feito

O que existe hoje, uma linha por área. Ao terminar um incremento, ajuste a linha da área (ou crie uma); o detalhe vai no commit e as regras no `decisions.md`.

- **Base**: workspace Rust (`engine`, `api`, `migrate`), Next 16 com next-intl (pt, es), Bun, Biome, recarga automática da API, código por domínio dos dois lados.
- **Contas**: login com Discord e com username e senha, vincular e desvincular, nome exibido; sessões e tentativas de senha no Redis, com rotas para listar e encerrar sessões.
- **Comunidades**: criar (até 3 como owner), convites, papéis, apelido, sair, remover, transferir a posse; membro inativo mantém o histórico; página da pessoa com os torneios dela na comunidade.
- **Duelo de ponta a ponta**: rascunho, traduções, visibilidade, inscrição (botão, organizador, link), check-in com abertura automática, seeding (sorteio com seed, ordem manual), iniciar e desfazer; eliminação simples e dupla; resultado mapa a mapa com confirmação por tempo, contestação, correção, anulação e W.O.; desistência com W.O. automático; encerramento com classificação.
- **"Seus torneios"** na home (`GET /me/tournaments`): situação de quem vê e a próxima partida.
- **Banco** que recusa estado inválido e órfãos (checks, FKs compostas, triggers no commit), com testes por SQL direto em `tests/api/schema/`.
- **Interface** no Carbon Design System (alvo em `docs/design.md`): `shared/ui` própria, abas por sub-rota, tema claro/escuro, revisões de uso (carregamento, foco, contraste, celular).
