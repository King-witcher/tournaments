# Torneios: criação, inscrição, check-in e seeding

Formatos e times: `formats.md`. Resultados: `matches.md`. Veto, campeões e mapas: `veto.md`. A interface do formulário e da página: `tournaments-ui.md`.

## Criação e edição

- Endereço `/{locale}/{tenant}/{torneio}`. Na interface o slug se chama "Id". Mesmas regras do slug de tenant (sugerido, ajustável na criação, a-z/0-9/hífen, 3 a 32), único dentro do tenant e **não muda**; palavras reservadas para as rotas do tenant (`new`, `members`, `invites`, `settings`, `ranking`...).
- **Só membros** veem torneios; quem é de fora recebe 404 `tenant.not_found`. Rascunho só para organizer e owner (para player, 404 `tournament.not_found`).
- **Inscrição aberta ou fechada** (substitui a visibilidade pública/privada: o anúncio de um torneio privado no Discord mostrou que o ganho real é controlar quem entra, não esconder): todo torneio publicado aparece para todos os membros, em todas as telas, no Discord e nos avisos. `admission` é `open` (padrão: botão de se inscrever) ou `invite_only` ("só por link ou pela organização": sem botão; link e inscrição manual continuam; o botão recusado responde 409 `tournament.invite_only`). Muda até o torneio encerrar.
- Organizer e owner criam e editam. **Criar já publica** (abre a inscrição) ou, em "Criar como rascunho", gera um rascunho; só rascunho pode ser excluído. A criação recebe as traduções e o `publish` num POST só, numa transação (antes, uma falha na publicação deixava o rascunho e tentar de novo dava `tournament.slug_taken`). Erro de texto numa tradução volta com `params.locale`.
- **Vagas obrigatórias** (2 a 1024). A checagem de campeões na criação usa o pior caso para esse número (o pior caso não diminui com mais participantes) e roda de novo ao gerar a chave com os inscritos reais. As vagas fecham a inscrição quando lotam.
- **Pool de campeões**: tabela global `champions` (o Wrath, que não existe no jogo, saiu); o pool do torneio é o conjunto ativo. Excluir campeões por torneio fica para depois.
- Formato padrão do formulário de duelo: eliminação dupla **sem** reset, md3 nas partidas e md5 na grande final.
- Edição: nome, descrição, regras, traduções e tipo de inscrição até o torneio encerrar. Formato, vagas, regra de campeões, data de início e janela de check-in até ele começar; a checagem roda de novo a cada mudança e ao publicar. Vagas abaixo dos inscritos são recusadas (`tournament.max_participants_below_registered`).
- **Data de início** (`starts_at`, UTC) **opcional em qualquer status**: dá para publicar sem data e pôr, mudar ou tirar até começar. Sem data o check-in não abre sozinho (o organizador usa "abrir o check-in antes da hora"), não há aviso de hora de iniciar, e a UI mostra "sem data"; "Seus torneios" ordena os sem data depois (assumido). Tirar a data com o check-in aberto não o fecha. O organizador digita no próprio fuso e cada um vê no seu.
- **Data passada, ou mais perto do que a janela de check-in, é erro, não aviso** (o check-in abriria na hora, sem aviso aos jogadores): `tournament.starts_at_past` ou `tournament.starts_at_too_soon {minutes}` ao **criar**, ao **editar a data** (só quando ela muda: editar outra coisa de um torneio cuja data já passou é permitido) e ao **publicar** rascunho com data vencida; sem check-in, só não pode ser passada (`Schedule::check_start`). O campo faz a mesma conta ao digitar e bloqueia o envio.
- **Check-in**: janela por torneio, em minutos antes do início (padrão 30; API aceita 5 a 1440; o formulário oferece 15, 30, 45 e 60), ou **sem check-in**: não existe o status `check_in` e a inscrição vai direto para em andamento.
- Idioma original fixo. Tradução: uma por idioma diferente do original, com nome obrigatório; descrição ou regras vazias mostram as do original. A API devolve original e traduções; o front escolhe pelo idioma de quem vê.
- Descrição e regras em **texto simples**: quebras de linha e recuos ficam (`UserText`, `white-space: pre-wrap`), nada é formatado (o Markdown saiu).
- Erro com números sai como `{"error": "champions.pool_exhausted", "params": {"required": 32, "available": 17}}`, e o front interpola na tradução.

## Ciclo

inscrição → check-in → em andamento → encerrado.

- O check-in **abre sozinho** em `starts_at - janela`, avaliado pela API ao ler o torneio (sem job): a primeira leitura depois da hora grava a transição, com guarda contra mudança concorrente. O relógio fica na API, nunca no `engine`. Só anda para a frente: adiar o início com o check-in aberto não o fecha.
- O **início é sempre manual**: organizer ou owner clica "Iniciar" (do check-in, ou da inscrição, sem check-in). Também podem **abrir o check-in antes da hora** e **desfazer o início** (volta para o check-in, ou para a inscrição sem check-in). Desligar o check-in durante o check-in volta para a inscrição; ligar de novo com a abertura no futuro mantém a inscrição, e os check-ins feitos ficam.
- **Inscrição aberta até o início**: o check-in não fecha a inscrição. Durante o check-in, quem se inscreve (botão, link ou organização) **já entra com o check-in feito** (o banco exige: `participants_check_in_on_entry`; na da organização, `checked_in_by` é quem inscreveu). As vagas continuam fechando a inscrição quando lotam. No TDM, quem entra depois de os times estarem formados fica de reserva.
- **"Reabrir a inscrição" saiu** (assumido): com a inscrição aberta durante o check-in, só servia para fechar o check-in, e desligar e ligar o check-in na edição já faz isso. Saíram a rota, o botão e a coluna `registration_reopened_at`: a abertura automática vale sempre que a janela abre e o status ainda é inscrição.
- **Check-in** se faz e desfaz só durante o check-in: o próprio inscrito, e organizer/owner para qualquer um (fica gravado quem marcou). Quem não fez continua inscrito (histórico), marcado sem check-in, mas fora do seeding e da chave. Depois de iniciar, inscrições e check-ins congelam.
- Publicar pede confirmação: publicado, não volta a rascunho nem pode ser excluído.

## Inscrição

Sempre de membros do tenant, por três caminhos:

- **botão**: o próprio membro, de qualquer papel (organizer e owner também: papéis cumulativos, rota com `TenantMember<Player>`), só com `open` e até o início;
- **organizador**: inscreve qualquer membro, em qualquer tipo de inscrição, do rascunho até o início;
- **link de inscrição**: um convite do tenant que também inscreve. Mesma tabela (`tenant_invites.tournament_id`), mesmo endereço, multiuso, revogável pela mesma rota. Sem data: vale até o início (também no check-in) e pode ser gerado no rascunho. Aceitar entra no tenant como player (quem já é membro mantém papel e apelido) e inscreve **numa transação só**: sem vaga ou fora da inscrição, não entra em nada.

Lotou, fecha (`tournament.full`). O inscrito sai sozinho até o início, também durante o check-in (no TDM com os times formados, sai do time e o time fica incompleto até a organização ajustar); depois de iniciar só o organizador tira (`tournament.participants_locked`). Inscrever de novo quem já está inscrito não dá erro.

## Seeding

Na interface, **"Ordem da chave"** ("seeding" sozinho não diz nada a quem não organiza torneios). Só organizer e owner, antes de iniciar; lista os elegíveis (com check-in; sem check-in no torneio, todos) na ordem que vale.

- "Sortear" gera uma seed u64 na API, grava no torneio e embaralha pelo `engine` (`seeding::random_order`, ChaCha8: mesma seed e mesmos elegíveis dão a mesma ordem em qualquer versão; um teste fixa).
- O seeding ordena **times** (no duelo, um time de um jogador): o ajuste manual manda a ordem completa de `team_ids` (`PUT ./seeding`), que precisa ter exatamente os times elegíveis (todos os jogadores com check-in; `seeding.order_invalid` se alguém mudou no meio). Seeds em `teams.seed`.
- **Ao iniciar**: vale a ordem gravada; elegíveis sem seed vão para o fim, na ordem de inscrição; sem seed nenhum, sorteia (e grava). Seeds compactados em 1..N; mínimo de 2 elegíveis (`tournament.not_enough_eligible`).
- **Por rating**: "Ordenar por rating" só preenche a ordem na tela (maior primeiro; sem rating no fim; empate na ordem anterior); cada linha mostra o rating do time (média dos jogadores, o inicial para quem não tem rating, "sem rating" quando nenhum tem); a organização ainda salva pela rota manual.
- A tela explica que os de cima pegam os de baixo na primeira rodada e os dois primeiros só se cruzam no fim; sem seed nenhum, avisa que a ordem mostrada não vale até salvar. Cada posição é um campo numérico (digitar e Enter leva para lá, os outros descem; setas para o ajuste fino). Arrastar ficou de fora: impreciso numa lista que rola no celular.

## i18n

- A API devolve **códigos**, nunca texto; status, formatos e fases são enums traduzidos no front. Datas em UTC no banco, exibidas no fuso de quem vê.
- Seletor de idioma no cabeçalho, mantém a página; cada idioma aparece no próprio nome ("Español" também para quem lê em português).
- Espanhol: a chave do torneio é sempre **llave** (mais comum na Argentina que "cuadro").
