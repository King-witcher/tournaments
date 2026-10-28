# Roadmap

O que fazer e em que ordem. Regras e motivos ficam em `decisions.md`; o detalhe de cada incremento, no `git log`.

## Próximo

Um incremento por vez, cada um com migration, rotas (OpenAPI), testes `sqlx::test`, tipos regenerados no front e tela mínima.

1. **Veto** de mapas e pick/ban de campeões ao vivo, com a regra de não repetição. Entra entre a partida ficar pronta e o envio: o pool por torneio sai de `maps`, e o envio de cada mapa passa a seguir o veto. Partida de quem desistiu não chega ao veto: vira W.O. ao ficar pronta. O aviso "veto começou" entra no `outbox` como os outros.
2. **Prints do placar** (S3 genérico, URL assinada, WebP, apagar 30 dias após o fim).
3. **Rating** (Elo, configurável por tenant), por mapa a partir de `match_games` (série sem detalhe conta pelos mapas vencidos). W.O. não conta.

## Depois

- Outros formatos: FFA por pontos/frags, 2v2/3v3 com formação de times, round robin, suíço.

## Pendências

O que ficou de fora de incrementos já feitos, com o que falta.

- **Conferir a limpeza da `shared/ui` no navegador** (só passou por tipo, lint e build): `FormModal` de "inscrever alguém" (foco, Enter, foco de volta), `PassiveModal` do link de inscrição, anular mapa com nota, `FormActions` lado a lado, `RadioGroup` do mapa a 390px, listas vazias com a ação na barra, foco no campo recusado (apelido, comunidade), o painel de comunidades no cabeçalho (carregando na primeira abertura, foco no primeiro item quando a lista chega, fechar no link da comunidade atual, nome longo).
- **Foco depois das ações da organização nas listas** (inscrever, tirar, check-in, desistência, seeding, convites e links): o item some ou muda de lugar, e o alvo certo é a lista.
- **Publicar com o check-in já aberto** leva direto ao check-in (hoje com aviso). A alternativa, publicar suspender a abertura automática, muda a API: decidir com o usuário.
- **Excluir conta e excluir comunidade**: regra para o usuário decidir ("Consistência do banco" no `decisions.md`).
- **Aviso fora do site** (Discord, email): o provedor de email já existe (Resend, `mail/`); falta o bot do Discord e decidir o que vale aviso por email (a cota grátis é de 100 por dia).
- **Email, o que ficou para depois**: captcha (Cloudflare Turnstile) no cadastro e na recuperação se aparecer abuso; avisar o email antigo quando o da conta muda; remover o email da conta; webhook de bounce do Resend para não insistir em endereço que não existe. Conferir no navegador o cadastro, a recuperação e a troca de email (só passaram por testes de rota, lint e build) e o primeiro envio real pelo Resend.
- **Conferir o tempo real no navegador**: conferido de ponta a ponta pelo agente (atualização ao vivo, toast, push real pelo Chrome); faltam o clique numa notificação, o iPhone com o site instalado, o `access_lost` e a reconexão depois de queda. Também falta a aba escondida (só passou por tipo, lint e build): o título com os avisos e a volta dele, os toasts ao voltar, o som sem push e a reconexão com a aba escondida (derrubar a API com a aba em segundo plano e subir de novo). Em produção: chaves VAPID no Render (`bun scripts/vapid-keys.ts`); sem elas, só o aviso com o site aberto.
- **Avisos: limites conhecidos**: o nome do torneio vai no idioma original (não no da inscrição); quem perde a inscrição num torneio privado continua recebendo os `changed` dele até recarregar; sessão que expira ou é encerrada em outro aparelho não tira a inscrição de push do navegador (sair pelo menu tira); o manifest não é traduzido; durante a troca de instâncias num deploy, um evento só chega aos sockets da instância que o despachou.
- **Tirar um W.O.**: série decidida por W.O. não reabre, então corrigir a partida anterior a ela só se resolve desfazendo o início. Criar a rota se aparecer na prática.
- **Pendências da organização em todas as comunidades**: "Seus torneios" só mostra as partidas de quem joga; contestações a resolver ficam na página de cada torneio.
- **Histórico completo de quem vê**: "Seus torneios" mostra só os 5 encerrados mais recentes; a lista inteira vai para uma página de perfil, com o rating.
- **Home de quem não entrou** mostra o esqueleto por um instante (ler cookies fica dentro do `<Suspense>` com Cache Components); tirar pede uma home estática para quem não tem cookie, decidida no proxy.
- **"Não encontrado" responde 200** (o conteúdo chega por streaming depois do status); corrigir pede `notFound()` fora do `Suspense`, o que mexe na estrutura das páginas.

## Feito

O que existe hoje, uma linha por área. Ao terminar um incremento, ajuste a linha da área (ou crie uma); o detalhe vai no commit e as regras no `decisions.md`.

- **CI** (`.github/workflows/ci.yml`): backend (fmt, clippy com `SQLX_OFFLINE=true`, migrations, testes, `.sqlx/` igual ao que as queries geram) e web (lint, build), só o lado que mudou em pull request e os dois em push na `main`.
- **Base**: workspace Rust (`engine`, `api`, `migrate`), Next 16 com next-intl (pt, es), Bun, Biome, recarga automática da API, código por domínio dos dois lados.
- **Contas**: login com Discord e com username e senha, vincular e desvincular, nome exibido; cadastro com senha só com email confirmado pelo link (forma canônica, sem descartáveis), recuperação de senha e troca de email por link (Resend); sessões, tentativas de senha, cadastros por confirmar e links no Redis, com rotas e aba Sessões na conta (navegador e sistema, desde quando, a atual marcada, encerrar uma ou as outras).
- **Comunidades**: criar (até 3 como owner), convites, papéis, apelido, sair, remover, transferir a posse; membro inativo mantém o histórico; página da pessoa com os torneios dela na comunidade.
- **Duelo de ponta a ponta**: rascunho, traduções, visibilidade, inscrição (botão, organizador, link), check-in com abertura automática, seeding (sorteio com seed, ordem manual), iniciar e desfazer; eliminação simples e dupla; resultado mapa a mapa com confirmação por tempo, contestação, correção, anulação e W.O.; desistência com W.O. automático; encerramento com classificação.
- **"Seus torneios"** na home (`GET /me/tournaments`): situação de quem vê e a próxima partida.
- **Tempo real e avisos** (regras em "Tempo real e avisos" no `decisions.md`): `GET /ws` com assinatura por comunidade e torneio e acesso conferido; eventos no `outbox` na transação de quem muda, despachados depois do commit; agendador de 10 s (confirmação por tempo, abertura do check-in, hora de iniciar); páginas recarregam sozinhas (juntando avisos, sem recarregar aba escondida); avisos de check-in, partida pronta, mapa enviado, contestação e hora de iniciar, como toast com som na página e Web Push com o site fechado (service worker traduzido, manifest, aba "Avisos" na conta, convite no tile "Sua situação"); com a aba escondida, o título conta os avisos, o toast aparece ao voltar, o som toca se não houver push e a conexão continua reconectando.
- **Banco** que recusa estado inválido e órfãos (checks, FKs compostas, triggers no commit), com testes por SQL direto em `tests/api/schema/`.
- **Interface** no Carbon Design System (alvo em `docs/design.md`): `shared/ui` própria, abas por sub-rota, suas comunidades no cabeçalho, tema claro/escuro, revisões de uso (carregamento, foco, contraste, celular); formulário de torneio com grupos fechados, revisão antes de publicar, abas na edição e por idioma; regras em texto simples, editáveis na própria aba.
