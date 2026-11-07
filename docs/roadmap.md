# Roadmap

O que fazer, em que ordem, e o que falta conferir. O que já existe, com as regras e os motivos, por área: `docs/decisions/` (índice em `decisions.md`); o detalhe de cada incremento, no `git log`.

## Próximo

- **Regras da organização em Markdown**: decidido com o usuário em 06/11, aguardando liberação. `TextArea` com abas Escrever/Visualizar e um subconjunto de Markdown (títulos, listas, negrito, links) renderizado sem HTML, com títulos no estilo dos blocos de "Como funciona"; mesma coluna `rules` (o texto atual continua válido) e as traduções também. Imagens só pelo nosso upload, no bucket público/CDN, com limite por torneio, removidas pela fila `image_deletions` quando saem do texto e com cache curto (cerca de 1 h) em vez de invalidar o CDN. Detalhe em `docs/decisions/tournaments-ui.md` ("Resumo e regras") e `images.md`.

Demais itens de "Depois" ainda não foram escolhidos. Um incremento por vez, cada um com migration, rotas (OpenAPI), testes de rota (`db::test_db`), tipos regenerados no front e tela mínima.

## Depois

- Outros formatos: FFA por pontos/frags, playoffs (os N melhores da tabela do round robin ou do suíço numa eliminação) e grupos.
- A capa do torneio nos avisos do Discord (embed com `image`; hoje a mensagem é só texto, `discord/render.rs`), sem precisar de página pública.

## Pendências

Cada uma diz o que falta e onde. As conferências no navegador e nos ambientes até 06/11 foram feitas pelo usuário; o que segue são limites conhecidos, decisões em aberto e o que fica para depois.

- **Conta com provedor e sem senha**: a sessão roubada ainda troca o email e desvincula o provedor (`auth.md`, "Conta só com email"). Decidir se a prova do email de agora vale também aí.
- **Custo por requisição**: "Seus torneios" (`my_tournaments/handlers.rs`) faz uma lista por comunidade e, por torneio na chave, `matches::place` (a fase inteira) e `team_of`: N+1 de verdade. Medir no Render o tempo do clique depois dos cortes de 01/11 (`stack.md`, "Idas ao banco") e só então decidir sobre `test_before_acquire`.
- **"Não encontrado" responde 200** (soft 404): com o Cache Components, a casca estática (layout do idioma, cabeçalho, esqueletos) sai com o 200 antes de qualquer busca, e `notFound()` fora do `Suspense` não muda isso (medido em 06/11). As páginas de entidade mostram o código `*.not_found` da API em `PageError`. 404 de verdade só checando no `proxy.ts`, em pedido de documento com sessão (protótipo funcionou): uma ida a mais à API por carga completa e o esqueleto esperando a ida Vercel → Render. Por ora não vale: tudo é `noindex` e pede login, menos convite e sala de veto.
- **Discord bloqueando o Render** (`auth.md`): a saída tem teto e pausa, mas o bloqueio por vizinho continua possível. Acompanhar no log `provedor falhou na troca do code` com `cloudflare=true`; se voltar, escolher entre proxy de IP fixo, relay na Vercel e IP dedicado.
- **Sessões em memória**: 30 dias depois de chegar à produção, tirar a leitura do formato antigo (`from_old_format`, em `auth/kv/sessions.rs`).
- **Interface, limites conhecidos**: o `Callout` e o `ActionableNotification` da `shared/ui` usados direto ainda param em 288 px abaixo do `md` (o `Notice` e os toasts já ocupam a largura); o toast cobre o topo dos outros painéis do cabeçalho (comunidades, pessoa); o ícone de busca recolhida do `ExpandableSearch` diz "Search" (o Carbon 1.119 não tem prop para isso); os comentários do OpenAPI ainda dizem "mapa" para partida.
- **`useFormValidity` com `NumberInput`**: não serve ao placar (o `NumberInput` controlado atualiza o campo num commit depois do render e o botão ficava um passo atrás); o mesmo pode valer em `rating-settings-form` e onde houver `NumberInput` com `canSubmit`.
- **TDM**: a regra de iniciar repete a do quadro de estado quando falta time completo; as pendências da organização não falam de times incompletos. Depois: mudar o pool de campeões por torneio.
- **Ordem das escolhas** (`veto.md`): a substituição não reinicia o prazo dos picks, e o substituto de uma série já longe do draft é sorteado na hora, sem chegar a escolher (`teams/roster.rs`, `VetoLog::clock`). Desfazer depois de uma troca de elenco tira o último passo gravado, que pode ser de outro lado (`Event::Undo`). A substituição que deixa os dois lados prontos só começa a série na próxima leitura ou no próximo despertar do relógio.
- **Veto**: o mapa que o organizador lança fora do veto não muda os mapas dos jogos seguintes (o veto de mapas é da série inteira).
- **Disputa de posições**: não há teste de rota da disputa com veto (o veto é o mesmo dos outros formatos).
- **Menor rating**: a tela de configuração e a Classificação com o critério não foram vistas no navegador (só lint e build). O rating congelado não aparece em coluna, só na explicação dos desempates; e o critério só vale depois da primeira série decidida (`formats.md`, "Desempate por fase"): confirmar com o usuário.
- **Round robin e suíço**: um round robin de 32 mostra 496 cartões numa página: recolher as rodadas encerradas se pesar. "Seus torneios" entre duas rodadas talvez deva dizer "esperando a rodada seguinte".
- **Empate no fim da fase** (`formats.md`): limitar a pendência às N primeiras posições (hoje, todo grupo empatado); série de desempate dentro do site, quando houver fases ligadas (um empate entre dois viraria Disputa de posições em md1); lembrete à organização se a pendência ficar parada (hoje, um aviso só). Por conferir no navegador: o select nas Configurações, o bloco "Empates a decidir" no celular e no teclado, e os textos em es e en.
- **Tirar um W.O.**: série decidida por W.O. não reabre, então corrigir a partida anterior só se resolve desfazendo o início. Criar a rota se aparecer na prática.
- **Endereço e excluir torneio**: a aba sem socket na hora da troca cai no "não encontrada" ao recarregar. A notificação de uma comunidade excluída continua com o endereço dela, que outra pode pegar (`tenants/deletion.rs`; os torneios excluídos já limpam).
- **Avisos, limites conhecidos**: o nome do torneio no toast do site aberto vai no idioma original; sessão que expira ou é encerrada em outro aparelho não tira a inscrição de push (sair pelo menu tira); o manifest não é traduzido; na troca de instâncias num deploy, um evento só chega aos sockets da instância que o despachou; as pendências da organização na home não recarregam quando só o nome do torneio ou o apelido de quem contestou muda.
- **Histórico completo de quem vê**: "Seus torneios" mostra só os 5 encerrados mais recentes; a lista inteira vai para uma página de perfil, com o rating.
- **Rating, depois**: manter o `confirmed_at` original na correção de um mapa; desfazer um ajuste; ranking ligado/desligado e rating de FFA.
- **Por mapa**: a regra de quais partidas contam é assumida (`ratings.md`). Não há índice por `(tenant_id, user_id)` em `match_game_players`: a consulta parte dos torneios da comunidade para usar o de `(torneio, jogador)`; se pesar, medir com `explain` e então criar o índice.
- **Tile de mapa duplicado**: `features/maps/components/map-tile.tsx` (imagem, nome e uma linha, usado no "Por mapa") repete o `veto-map-tile.tsx` de `features/matches` (que ainda tem o estado de banido e o botão). Unificar: o `VetoMapTile` passa a usar o `MapTile`.
- **Screenshots**: se aparecer abuso na leitura (a URL vale até 2 horas, para qualquer membro, e cada GB de saída é cobrado), CloudFront ou R2 (`S3_ENDPOINT`). A miniatura é a imagem inteira (até 2 MB): se pesar no celular, gerar miniatura no navegador. Screenshot anexado ao próximo mapa que não chega a ser enviado fica guardado e invisível até a rotina dos 30 dias; o de um mapa cujos lados mudaram por correção continua no mapa (a organização remove).
- **Email, depois**: captcha (Turnstile) se aparecer abuso; webhook de bounce do Resend; aviso de login novo (fora pela cota de 100 por dia).
- **Discord, fora por ora**: webhook por torneio ou canal, bot e DM.
- **Apresentação** (`/` sem sessão): as capturas mudam à mão quando a tela delas mudar (o script fica fora do git).
