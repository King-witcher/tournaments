# Torneios: criação, inscrição, check-in e seeding

Formatos e times: `formats.md`. Resultados: `matches.md`. Veto, campeões e mapas: `veto.md`. A interface do formulário e da página: `tournaments-ui.md`.

## Criação e edição

- Endereço `/{locale}/{tenant}/{torneio}`. Na interface o slug se chama "Endereço". Mesmas regras do slug de tenant (sugerido, ajustável na criação, a-z/0-9/hífen, 3 a 32), único dentro do tenant; palavras reservadas para as rotas do tenant (`new`, `members`, `settings`, `ranking`...).
- **O endereço muda só no rascunho** (do usuário, 03/11): pela organização, com a validação da criação e a mesma nota de consequência da comunidade (`tenants.md`) antes de salvar; o antigo fica livre na hora. Depois de publicar, 409 `tournament.slug_locked` (o banco também recusa, `tournaments_slug_after_draft`), e o formulário mostra o campo desabilitado, com a linha "O endereço não muda depois de publicar. Só o nome muda.", para quem procura como mudar entender. Voltar a rascunho destrava o endereço. Nas notificações e na fila do Discord não se reescreve nada: aviso de um torneio que voltou a rascunho é de quem já saiu da inscrição e leva a uma página que essa pessoa não vê. Link de inscrição continua valendo (usa token).
- **Só membros** veem torneios; quem é de fora recebe 404 `tenant.not_found`. Rascunho só para organizer e owner (para player, 404 `tournament.not_found`).
- **Inscrição aberta ou fechada** (substitui a visibilidade pública/privada: o anúncio de um torneio privado no Discord mostrou que o ganho real é controlar quem entra, não esconder): todo torneio publicado aparece para todos os membros, em todas as telas, no Discord e nos avisos. `admission` é `open` (padrão: botão de se inscrever) ou `invite_only` ("só por link ou pela organização": sem botão; link e inscrição manual continuam; o botão recusado responde 409 `tournament.invite_only`). Muda até o torneio encerrar.
- Organizer e owner criam e editam. **Criar já publica** (abre a inscrição) ou, em "Criar como rascunho", gera um rascunho. Excluir: `deletion.md`. A criação recebe as traduções e o `publish` num POST só, numa transação (antes, uma falha na publicação deixava o rascunho e tentar de novo dava `tournament.slug_taken`). Erro de texto numa tradução volta com `params.locale`.
- **Vagas** (2 a 1024; padrão 16). A checagem de campeões na criação usa o pior caso para esse número (o pior caso não diminui com mais participantes) e roda de novo ao gerar a chave com os inscritos reais. As vagas fecham a inscrição quando lotam.
- **Pool de campeões**: tabela global `champions` (o Wrath, que não existe no jogo, saiu); o pool do torneio é o conjunto ativo. Excluir campeões por torneio fica para depois.
- **Os padrões de um torneio novo são da API**: `POST` aceita o corpo mínimo (endereço, idioma, nome e modalidade) e preenche o resto: 16 vagas, eliminação dupla **sem** reset, md3 nas partidas e md5 na grande final, 1 ban por série, espelho permitido, sem repetição de campeão, check-in ligado, 10 min para contestar, veto de mapas e draft de campeões a 30 s, e, no TDM, o preset Balanceado (`TeamRules::BALANCED`: o capitão veta e bane, cada jogador escolhe o próprio campeão, qualquer um envia; do usuário, 06/11, no lugar do capitão em tudo; `EVERYONE` é só o padrão das colunas, de quem já existia). Motivo: a criação passa a ser curta e o resto se configura no rascunho, e o padrão em dois lugares divergia (o `team_rules` ausente dava "todos os jogadores" na API e "capitão" no formulário). Campo a campo ausente de `champion_rules` também cai no padrão (`ChampionRulesInput`); o PATCH não mudou.
- Edição: nome, descrição e as traduções deles **também depois de encerrar** (assumido, 03/11: um erro de digitação ficava para sempre; a capa já mudava). Regras e tipo de inscrição até o torneio encerrar (`tournament.finished`). Formato, vagas, regra de campeões, data de início e check-in até ele começar; a checagem roda de novo a cada mudança e ao publicar. Reenviar um valor igual ao gravado não é mudança, em nenhum estado. Vagas abaixo dos inscritos são recusadas (`tournament.max_participants_below_registered`).
- **Encerrado, as traduções** (assumido): o `PUT` aceita nome e descrição, com as regras iguais às gravadas (vazias, se a tradução é nova); o `DELETE` continua, porque tirar uma tradução só volta o idioma para o original, e as regras não mudam por aí. O formulário abre com o motivo no topo de cada aba e os outros campos desabilitados.
- **Data de início** (`starts_at`, UTC): **opcional e só informativa** (do usuário, 09/11). Aparece nos tiles, no cabeçalho, na prévia do convite e na ordem das listas ("Seus torneios" põe os sem data depois, assumido), no fuso de quem vê, e pode estar no passado. Nada acontece por data: o check-in não abre, ninguém é avisado e nada recusa ou bloqueia publicar. O único efeito é o tile "hora de iniciar" na home da organização, enquanto a data passada não vira início. O organizador digita no próprio fuso.
- **Check-in** (do usuário, 09/11): **liga ou desliga por torneio** (`tournaments.check_in`, no modelo de `tournament_stages.check_in`; ligado por padrão). Sem janela em minutos: abre só pelo botão. Desligado, não existe o status `check_in` e a inscrição vai direto para em andamento.
- Idioma original fixo. Tradução: uma por idioma diferente do original, com nome obrigatório; descrição ou regras vazias mostram as do original. A API devolve original e traduções; o front escolhe pelo idioma de quem vê.
- Descrição em **texto simples**: quebras de linha e recuos ficam (`UserText`, `white-space: pre-wrap`), nada é formatado. As regras da organização voltaram a aceitar Markdown em 07/11, num subconjunto sem HTML (`tournaments-ui.md`, "Resumo e regras").
- Erro com números sai como `{"error": "champions.pool_exhausted", "params": {"required": 32, "available": 17}}`, e o front interpola na tradução.

## Ciclo

rascunho → inscrição → check-in → em andamento → encerrado.

- **Tudo é manual** (do usuário, 09/11): organizer ou owner clica. Nada anda por data, nem ao ler nem no agendador. Antes o check-in abria sozinho em `starts_at - janela` e havia aviso de "hora de iniciar" (notificação, push e Discord): saíram, com `start_reminded_at`, `tournament.starts_at_past`, `starts_at_too_soon` e `check_in_minutes_invalid`. Fica só o tile na home da organização.
- **Rascunho é só configuração, sem inscritos** (do usuário, 09/11): ninguém entra, nem pela organização; a aba Inscritos e o link de inscrição não aparecem. O banco recusa nos dois sentidos (`participants_not_in_draft`, `tournaments_draft_without_participants`). **Publicar é abrir as inscrições.**
- **Voltar para rascunho** (do usuário, 09/11; `DELETE .../publish`): só com status `registration` e nenhum inscrito (409 `tournament.not_registration` ou `tournament.has_participants`). Destrava o endereço. Fica no ⋯; com inscritos, o item diz o motivo em vez de abrir a confirmação (assumido). O link de inscrição de um rascunho responde como link inexistente (404 `invite.not_found`), na prévia e no aceite, sem mostrar o torneio.
- **Iniciar**: do check-in, ou da inscrição, sem check-in. Também dá para **desfazer o início** (volta para o check-in, ou para a inscrição sem check-in).
- **Abrir e fechar o check-in**: "Abrir check-in" é o próximo passo do cabeçalho e avisa os inscritos. "Fechar check-in", no ⋯ (assumido, 09/11; `DELETE .../check-in/open`), volta para a inscrição, para quando se abriu por engano; os check-ins feitos ficam. Desligar o check-in na edição durante o check-in também volta para a inscrição.
- **Inscrição aberta até o início**: o check-in não fecha a inscrição. Durante o check-in, quem se inscreve (botão, link ou organização) **já entra com o check-in feito** (o banco exige: `participants_check_in_on_entry`; na da organização, `checked_in_by` é quem inscreveu). As vagas continuam fechando a inscrição quando lotam. No TDM, quem entra depois de os times estarem formados fica de reserva.
- **Check-in** se faz e desfaz só durante o check-in: o próprio inscrito, e organizer/owner para qualquer um (fica gravado quem marcou). Quem não fez continua inscrito (histórico), marcado sem check-in, mas fora do seeding e da chave. Depois de iniciar, inscrições e check-ins congelam.
- Publicar pede confirmação, com a revisão: as inscrições abrem na hora, e depois da primeira inscrição ele não volta a rascunho.

## Inscrição

Sempre de membros do tenant, por três caminhos:

- **botão**: o próprio membro, de qualquer papel (organizer e owner também: papéis cumulativos, rota com `TenantMember<Player>`), só com `open` e até o início;
- **organizador**: inscreve qualquer membro, em qualquer tipo de inscrição, de publicar até o início;
- **link de inscrição**: um por torneio, no modelo do convite da comunidade (`tournaments.invite_token`, mesmo endereço; `tenants.md`, "Convites"). Organizer e owner copiam e trocam na aba Inscritos, de publicar até o início; quem já se inscreveu continua. Sem data: inscreve com a inscrição aberta (até o início, também no check-in), em qualquer tipo de inscrição. Aceitar entra no tenant como player (quem já é membro mantém papel e apelido) e inscreve **numa transação só**: sem vaga ou fora da inscrição, não entra em nada.

Lotou, fecha (`tournament.full`). O inscrito sai sozinho até o início, também durante o check-in (no TDM com os times formados, sai do time e o time fica incompleto até a organização ajustar); depois de iniciar só o organizador tira (`tournament.participants_locked`). Inscrever de novo quem já está inscrito não dá erro.

## Seeding

Na interface, **"Ordem da chave"** ("seeding" sozinho não diz nada a quem não organiza torneios). Só organizer e owner, antes de iniciar; lista os elegíveis (com check-in; sem check-in no torneio, todos) na ordem que vale.

- "Sortear" gera uma seed u64 na API, grava no torneio e embaralha pelo `engine` (`seeding::random_order`, ChaCha8: mesma seed e mesmos elegíveis dão a mesma ordem em qualquer versão; um teste fixa).
- O seeding ordena **times** (no duelo, um time de um jogador): o ajuste manual manda a ordem completa de `team_ids` (`PUT ./seeding`), que precisa ter exatamente os times elegíveis (todos os jogadores com check-in; `seeding.order_invalid` se alguém mudou no meio). Seeds em `teams.seed`.
- **Ao iniciar**: vale a ordem gravada; elegíveis sem seed vão para o fim, na ordem de inscrição; sem seed nenhum, sorteia (e grava). Seeds compactados em 1..N; mínimo de 2 elegíveis (`tournament.not_enough_eligible`).
- **Por rating**: "Ordenar por rating" só preenche a ordem na tela (maior primeiro; sem rating no fim; empate na ordem anterior); cada linha mostra o rating do time (média dos jogadores, o inicial para quem não tem rating, "sem rating" quando nenhum tem); a organização ainda salva pela rota manual.
- **Sem ordem salva, o início sorteia** (o usuário confirmou em 06/11, revisão 5): a ordem de inscrição não vira seed sem querer; "Salvar ordem" a mantém e "Sortear" sorteia na hora.
- A tela explica o que a ordem decide, pelo formato da primeira fase (na eliminação, os de cima pegam os de baixo na primeira rodada); sem seed nenhum, a lista aparece na ordem de inscrição, sem números (eles surgem quando a pessoa mexe na ordem), e o aviso diz que a chave é sorteada ao iniciar se não salvar. Cada posição é um campo numérico (digitar e Enter leva para lá, os outros descem; setas para o ajuste fino). Arrastar ficou de fora: impreciso numa lista que rola no celular.

## i18n

- A API devolve **códigos**, nunca texto; status, formatos e fases são enums traduzidos no front. Datas em UTC no banco, exibidas no fuso de quem vê.
- Seletor de idioma no rodapé, mantém a página; cada idioma aparece no próprio nome ("Español" também para quem lê em português).
- Espanhol: a chave do torneio é sempre **llave** (mais comum na Argentina que "cuadro").
- Inglês: a chave do torneio é **bracket** (winners bracket e losers bracket; códigos `W1.2` e `L1.2`).
