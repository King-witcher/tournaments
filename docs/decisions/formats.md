# Formatos, times e TDM

Regras no `engine` (`formats/`, `teams.rs`); a API grava e sabe a hora. "Assumido" = decisão minha, fácil de mudar; o resto foi decidido pelo usuário.

## Formatos

- Dois eixos independentes: **chaveamento** (eliminação simples, dupla, round robin, suíço, disputa de posições, lobbies FFA, série de pontos) e **partida** (melhor de N; no FFA, pontos por colocação ou soma de frags).
- Duelo e TDM (2v2, 3v3, 4v4) são o mesmo motor com `team_size` diferente.
- O torneio tem de 1 a 4 **fases** (`stages`), em sequência: cada uma recebe a classificação da anterior (ver "Fases ligadas").
- Fluxo: inscrição individual → check-in → formação de times → seeding → chave.
- Toda aleatoriedade (sorteios, timeouts) recebe seed ou RNG por parâmetro e a seed é salva, para ser reproduzível e auditável.
- Seeding: manual, aleatório (seed salva) ou por rating (ver `tournaments.md`).

## Times em todos os modos

Quem ocupa um lado da chave é um **time**, em todos os modos; no duelo, um time de um jogador, sem nada visível de diferente (a palavra "time" não aparece). A migration `20261119000000_teams` **apagou todos os torneios** em vez de convertê-los (produção sem usuários). O resto é assumido:

- **Tabelas**: `teams` (id, torneio, nome opcional, capitão, seed único por torneio, `withdrawn_at`/`withdrawn_by`) é o `ParticipantId` da chave; `participants` continua a inscrição individual (check-in, quem inscreveu) com `team_id` (nulo: sem time). Os lados de `match_results`, `match_games`, `match_events` e `match_veto_events` apontam para `teams`.
- **No duelo, o banco cria o time de um jogador** na inscrição (trigger `participants_solo_team`) e ele some com ela. Assim o seeding ordena times do mesmo jeito em todos os modos. Depois do início, nenhum time entra nem sai e o seed fica; nome e capitão ainda mudam.
- **Elegível para o seeding**: time em que todos os jogadores fizeram check-in. A ordem "de inscrição" dos sem seed é a de criação do time.
- **Fora do torneio** (leva ao W.O. automático): o time desistiu ou nenhum jogador dele continua na comunidade. **Desistência é do time** (`POST .../teams/{team}/withdrawal`). Quem sai da comunidade em torneio em andamento faz o time desistir se ninguém mais dele fica.
- **Escalação por mapa** (`match_game_players`): quem jogou cada mapa, por lado, é a fonte do rating. O banco grava quando o mapa entra ou muda de lados (trigger `match_games_lineup`), em todo caminho (jogador, lançamento e correção do organizador); no TDM, é quem está no time e na comunidade naquele momento (se ninguém está, como na correção de um mapa antigo, entram todos). O envio não pede escalação.
- **"Jogador da partida"** é membro do time de um dos lados. Os avisos (toast, Web Push, Discord) vão para os jogadores do time; o `opponent` dos avisos é o nome do time (sem nome, o apelido do capitão).
- **Nome exibido de um lado**, num helper do front: o nome do time; sem nome e com um jogador, o apelido ("Conta excluída" se for o caso); sem nome e com mais, "Time de {capitão}" / "Equipo de {capitán}". No time de um jogador o nome liga à página da pessoa, menos para quem saiu da comunidade (a página dela não abre).

## Round robin e suíço

A classificação da fase sai da tabela. Playoffs (os N melhores numa eliminação) são uma fase seguinte com corte (ver "Fases ligadas"); grupos (fases em paralelo) ficam para depois.

Decidido pelo usuário:

- **Suíço clássico**: número fixo de rodadas (padrão ⌈log2 N⌉, configurável), todos jogam todas; pareamento por pontos sem revanche; o bye vale vitória e vai para o pior colocado ainda sem bye; a rodada seguinte nasce quando a anterior termina.
- **Desempate**: no round robin, saldo de séries (ver "Séries pares e saldo de séries"); no suíço, pontos (séries vencidas e byes; a série é sempre ímpar e não empata) e **Buchholz** logo depois. Em seguida, nos dois, confronto direto → saldo de mapas → saldo de frags → seed. Quem continua empatado depois do saldo de mapas ou de frags **volta ao confronto direto**, só entre si: quem venceu a série fica à frente, em vez de a posição sair por frags ou seed. Desde 05/11 essa é a ordem padrão, escolhida por fase ("Desempate por fase").
- **Desistência**: no suíço, quem desiste **mantém os pontos** e não é mais pareado; no round robin, vira **W.O. em tudo que faltava**.
- **W.O. vale vitória sem perder mapa** (2 x 0 na md3, 3 x 0 na md5, no saldo de mapas; sem frags), para o vencedor não ficar atrás de quem jogou o mesmo adversário só pela ordem das partidas.
- **Ordem das séries no round robin**: a ordem das rodadas continua como padrão; quando o adversário da vez acabou de começar outra série, o site sugere um adversário livre, e a chave marca quem está jogando. Só indica, nunca bloqueia: uma série contestada ou abandonada prenderia o time. Cada time joga N − 1 séries em sequência (8 jogadores em MD3: cerca de 5 h sem nenhuma espera), então, em simulação, sair da ordem não encurta com todos presentes (6 jogadores: de 4,2 h para 4,9 h) e só ajuda em torneio grande quando alguém some (16 jogadores com pausas: de 16,0 h para 14,9 h).
- **Suíço sem pareamento contínuo**: a rodada continua esperando todas as séries, sem prazo por rodada nem aviso de atraso ao organizador. Em simulação, parear quem termina não encurta (16 jogadores: 3,8 h contra 3,9 h) e deixaria escolher o adversário atrasando o envio do resultado.

No `engine` (assumido):

- **Round robin**: cada par uma vez, método do círculo (Berger). N ímpar: cada um folga uma vez, e a folga **não vale ponto** (partida `bye`, a última da rodada, só para a tela). **Todas as partidas ficam prontas desde o início** (a interface agrupa por rodada). Até **32 participantes** (496 partidas; `format.too_many_participants {max}`).
- **Uma série** (melhor de N: ímpar; no round robin, também par) para todas as partidas da fase; não há final. Rodadas do suíço nulas = automático.
- **Suíço**: rodadas de 1 a 127 (`format.invalid_rounds {max, got}`), no máximo N − 1 com os N do início (com menos inscritos, valem N − 1, sem erro). Até **128 participantes** (pareamento medido em 4 ms na pior rodada).
- **Pareamentos gravados**: o `engine` gera o pareamento de cada rodada (`next_round`) quando todas as séries das rodadas pareadas estão decididas, e a API grava; o replay usa os gravados. Corrigir um resultado antigo muda a tabela, **nunca refaz rodadas pareadas**; uma correção que reabre série segura a rodada seguinte. Pareamento determinístico (sem sorteio; o seed desempata).
- **Como pareia**: na ordem da tabela, com todos os desempates. Dentro do grupo de mesma pontuação, metades do sistema holandês (seed 1 contra N/2 + 1...); o que sobra de grupo ímpar desce e enfrenta o melhor do grupo de baixo (simplificação do holandês). O lado `a` é o melhor colocado.
- **Sem pareamento sem revanche** (poucos ativos, muitas rodadas, desistências), a regra **cede só o mínimo**: o `engine` calcula quantas revanches são inevitáveis (emparelhamento máximo de Edmonds no grafo de quem nunca se enfrentou) e não passa disso; a inevitável cai nos de baixo. Escolhido no lugar de limitar rodadas na validação: desistências tiram gente no meio e nenhum limite fixo garante o pareamento.
- **Bye**: com ativos ímpar, entre os com menos byes, o pior colocado; se o bye dele obriga a mais revanches que o de outro, o seguinte de baixo para cima. Ninguém recebe dois enquanto houver ativo sem bye. Bye = vitória sem mapas nem frags, e não soma Buchholz.
- **Desistência no suíço**: quem está fora não é pareado nem recebe bye; a série pronta dele vira W.O. pela API. Com menos de 2 ativos, a fase termina.
- **Confronto direto**: mini-tabela só com as séries decididas entre os empatados, refeita entre os que continuam empatados; **só vale quando todos os empatados já se enfrentaram** (no suíço, quase nunca), senão quem venceu um passaria quem não enfrentou nenhum. No round robin, a mini-tabela é o saldo dentro do grupo; no suíço, as vitórias.
- **Buchholz**: soma dos pontos atuais dos adversários das séries decididas (inclusive W.O.), contando duas vezes o adversário de revanche.
- **Saldo de frags**: só desempata empatados que têm frags em todas as séries jogadas (série com mapa sem frags fica sem, em vez de contar 0); senão, vale o critério seguinte (no padrão, o seed). Comparar frags de séries diferentes daria vantagem a quem informa.
- **Classificação final**: uma posição por participante, sem empate, só quando a fase termina; antes, a tabela mostra a ordem atual e o critério que pôs cada linha à frente da seguinte, para a tela explicar o desempate.
- Identificador de partida `rr-<rodada>-<n>` e `sw-<rodada>-<n>`; folga e bye são a última da rodada. Orçamento de campeões (`worst_case_games`): round robin (N − 1) × melhor de N; suíço rodadas × melhor de N.

Na API e na tela (assumido):

- **Round robin e suíço na API** (`matches/stage.rs`, `rounds.rs`): o replay recebe os frags somados por série (sem frags se algum mapa veio sem), os pareamentos gravados e quem está fora. O `settle` grava o pareamento da rodada seguinte (e a rodada 1 no início), refazendo a fase até não haver rodada nova (`matches/rounds.rs`). Limite conhecido: quem saiu e voltou depois do fim conta de novo como ativo se uma correção reabrir o suíço.
- **`match.ready` no round robin**: como todas ficam prontas no início, avisar todas mandaria N − 1 avisos de uma vez (e ao Discord). Cada time recebe só a **série sugerida**, no início e quando termina uma série, a não ser que já esteja em outra; o adversário livre cuja vez também é essa recebe junto, mesmo que já a conhecesse, porque era o outro lado que ele esperava. A série reaberta avisa os dois lados. No suíço, os dois lados de cada partida de rodada nova, como na eliminação.
- **Sugestão no round robin** (`engine`, `round_robin::suggest`; `matches/progress.rs`): sem estado novo, tudo sai dos mapas enviados e do veto. A ordem de prioridade é a série em jogo, a chamada por um lado ("Pronto" no veto), a primeira aberta dos dois e o salto entre dois times cujos adversários da vez vão demorar (faltam ao menos 2 mapas, cerca de 30 min: o limite que não piorou o torneio na simulação). "Em jogo" é série com veto começado ou mapa enviado e ainda com mapa a jogar: quem só espera confirmação conta como livre. Em torneio sem veto, a série só aparece em jogo com o primeiro mapa enviado, cerca de 15 min depois de começar. Série abandonada continua marcada: é só indicação. A próxima partida de "Seus torneios" é a série com mapa do adversário a responder, senão a sugerida; com o adversário jogando, ela não entra em "Esperando você" e os cartões trocam a tarefa por "Adversário jogando".
- "Seus torneios" mostra "jogando" até o fim e depois "Terminou em Nº"; entre duas rodadas do suíço, quem já jogou fica sem próxima partida.
- Tela: a tabela em cima, os desempates que não foram pelos pontos em texto ("2º (X) à frente do 3º (Y) pelo confronto direto") e as partidas por rodada. O formulário limita as vagas ao teto do formato (32 e 128), com o erro no campo.

## Séries pares e saldo de séries

Do usuário (05/11): o round robin aceita séries pares além das ímpares (de início 2, 4 e 6; desde 06/11 qualquer N par, com o mesmo teto do ímpar, e o formulário oferece de 1 a 9), e a tabela dele ordena pelo **saldo de séries** em qualquer melhor de N: vitória +1, empate 0, derrota −1, no lugar dos pontos como primeiro critério. Assumido: as séries pares são só do round robin (eliminação e suíço recusam, `format.invalid_best_of`; a eliminação precisa de quem avança). Como a série par fecha: `matches.md`; o veto dela: `veto.md`.

- **A ordem final em md ímpar não muda.** Sem empate, todos jogam N − 1 séries (W.O. inclusive), então o saldo é `2 × vitórias − (N − 1)`: os mesmos grupos que as vitórias dão. Teste de propriedade em `round_robin/tests/props.rs`, com md1, md3, md5 e md7, de 2 a 16 jogadores. Só muda a ordem durante o torneio: quem jogou mais não fica à frente só por isso.
- **Sem configuração.** 2/1/0 dá a mesma ordem que o saldo: saldo = pontos − séries jogadas, e todos jogam o mesmo número. Só 3/1/0 mudaria a classificação; entraria com uma coluna nova na fase, sem converter resultados, porque a tabela é derivada.
- **O suíço fica como está**: pontos = vitórias + byes, Buchholz e o pareamento (`pair()` usa os pontos). Pelo saldo, quem desiste com 2-0 passaria quem foi até o fim com 3-2 (+2 contra +1). A mini-tabela do confronto direto dele também continua nas vitórias: com revanche (A vence B duas vezes, B vence C, C vence A), as duas contas dão ordens diferentes.
- **Confronto direto com empate**: a série empatada conta como confronto realizado e vale 0. Quem empatou entre si não se separa por ele e segue para o saldo de mapas.
- **Saldo de mapas**: a série empatada conta metade para cada lado, e o W.O. vale a maioria sem perder mapa. Na md2 ele é sempre o dobro do saldo de séries, então, na prática, decidem os frags; da md4 em diante desempata de verdade (na md4, 3 x 0 vale +3; 3 x 1, +2). Sem código especial para isso. Os frags da série empatada somam para cada lado (`SeriesFrags` é por lado).
- **Tela**: coluna "Saldo" com sinal no round robin, "Pontos" no suíço; na série par, o recorde em V–E–D. A explicação dos desempates ignora o primeiro critério.

## Desempate por fase

Do usuário (05/11):

- Nas fases de tabela, o organizador escolhe, para a fase, a **ordem** dos desempates e quais ficam **ligados**. Na eliminação não há desempate.
- **Fixos, fora da escolha**: o primeiro critério (saldo de séries no todos contra todos, pontos no suíço) e, por último, a ordem da chave (seed), para nunca sobrar empate.
- **Escolhem-se** confronto direto, saldo de mapas, saldo de frags, menor rating e, só no suíço, Buchholz. Desligar todos vale: decide a ordem da chave.
- **Padrão = a ordem de antes** (acima), que a migration `20261211020000_stage_tiebreakers` grava nas fases que já existem (testada numa cópia do banco de dev: 41 fases, 28 de torneios começados). Nenhuma tabela muda: o `replay_golden` passa sem mudar o esperado, com dois casos novos de ordem escolhida.
- **Menor rating** (06/11, revoga o "sem critério novo" de 05/11): entre empatados, fica à frente quem tem o rating menor. Quem tem o maior tinha a obrigação de vencer; empatar com alguém pior é mais derrota que vitória. Desligado por padrão: nenhuma tabela existente muda (o `replay_golden` passa sem mudar o esperado).

Assumido:

- **Onde fica**: `tournament_stages.tiebreakers text[]`, nula na eliminação e presente nas de tabela (`tournament_stages_shape`), só os valores que existem (cinco desde `20261215000000_lower_rating_tiebreaker`), cada um uma vez, numa lista simples (`tournament_stages_tiebreakers`), `buchholz` só no suíço (`tournament_stages_buchholz_swiss`). No `engine`, `Tiebreakers` com no máximo 5, de tamanho fixo, para o formato continuar `Copy`. Lista inválida: `format.invalid_tiebreakers`. Sem a lista no corpo, o padrão do formato.
- **Regras do motor que continuam**: a volta ao confronto direto vale para quem segue empatado depois de qualquer critério posto depois dele (mapas, frags, menor rating ou o Buchholz), e só com ele ligado; o saldo de frags só vale com os frags completos de todo o grupo, senão vale o critério seguinte.
- **O pareamento do suíço segue a tabela**: o desempate escolhido muda quem enfrenta quem (com só os frags, a rodada 2 de 6 jogadores sai com outros pares; teste `the_tiebreakers_change_the_pairing`). Rodada já pareada não muda.
- **Suíço sem Buchholz**: ele não é calculado (`buchholz` nulo na tabela) e a coluna some.
- **Trava**: depois do início, nada muda; o trigger que já congela a fase inteira (`tournament_stages_frozen_after_start`) cobre a coluna.
- **Menor rating, o do início**: ao iniciar um torneio com ele, cada time da chave grava `teams.rating_at_start`, a média dos jogadores em centésimos (inteiro, para comparar sem `f64`; com até 4 por time, médias diferentes nunca empatam), com o rating da comunidade de então (recalculado antes se há marca em `rating_dirty`) e o inicial para quem não tem, também em torneio que não conta para o rating. O rating muda a cada mapa e correção, e a tabela precisa sair igual no replay. Desfazer o início apaga; reiniciar grava de novo; a substituição não muda. O banco congela a coluna depois do início (`teams_started_frozen`) e só a aceita em time com seed. O `engine` recebe os ratings no replay (`StageHistory::ratings`).
- **Menor rating só depois da primeira série decidida da fase**: sem isso, a tabela do início e a rodada 1 do suíço sairiam pelo rating ao contrário, e não pela ordem da chave. Com algum empatado sem rating, vale o critério seguinte. Rating igual também passa adiante. Vale na tabela ao vivo e no pareamento do suíço, e a volta ao confronto direto vale depois dele como depois dos outros. A Classificação explica com os ratings ("pelo menor rating (1450 contra 1500)").

## Empate no fim da fase

Do usuário (06/11). Motivo: uma fase seguinte (fases ligadas, ou a Disputa de posições, que usa a classificação como ordem da chave) não pode começar com empate decidido pela sorte do seeding aleatório; e o padrão não pode surpreender quem não sabe que a opção existe.

- **Escolha por fase de tabela**, logo depois da lista de desempates: "Quando todos os critérios empatarem", com "Usar a ordem da chave" (**padrão**) ou "A organização decide".
- **Todos os empates que sobram** viram pendência; cada grupo também pode ser resolvido com "Usar a ordem da chave".
- **A fase espera**: com empate pendente, a fase e o torneio não terminam, mesmo com todas as séries decididas.

Assumido:

- **Só no fim**: durante a fase nada muda (tabela ao vivo e pareamento do suíço desempatam pelo seed). Com todas as séries decididas, o grupo que só o seed separaria é pendente, a não ser que haja decisão para exatamente esse conjunto de times.
- **No `engine`** (`formats/table/ties.rs`): `FinalTie` na configuração e as decisões (`TieOrder`) no `StageHistory`; a decisão vale pelo conjunto (a correção que muda quem empatou a deixa sem efeito; com o mesmo conjunto, continua; havendo duas, a última); a tabela mostra `organizer` como critério; os do grupo pendente ficam fora das colocações. O `replay_golden` não mudou.
- **Banco** (`20261215010000_stage_final_tie`): `tournament_stages.final_tie` (`seed` nas fases que já existiam), as decisões em `stage_tie_decisions` (quem, quando, motivo de até 280) com a ordem em `stage_tie_decision_teams`, só inseridas; `stage_pending_ties` marca a fase que espera, só com o torneio em andamento, e é o que a home da organização lê. "Usar a ordem da chave" também grava decisão. Desfazer o início apaga tudo.
- **API**: `POST .../ties` (ordem completa de um grupo pendente e motivo opcional; `tie.not_pending` se não bate) e `POST .../ties/seed` (todos pela ordem da chave), só organizer e owner. A fase termina pelo `settle`, como no fim normal, com o mesmo aviso de encerramento. A correção que cria pendência num torneio encerrado o reabre, como a que reabre série.
- **Aviso** `tournament.tie_pending` para organizers e owner (toast, Web Push e Discord, como a partida contestada), uma vez, quando a fase passa a esperar; de novo só se ela deixar de esperar e voltar.
- **Tela**: os empatados dividem a melhor posição, com "Empate a decidir"; a organização ordena cada grupo com setas (como os desempates do formulário), com motivo opcional e confirmação, porque a ordem gravada não muda. Quem está no grupo vê "Empate no 3º, a organização vai decidir"; os outros já veem a posição. O rating não muda (é por mapa).

## Fases ligadas

Um torneio em fases, uma depois da outra: o caso que motivou é todos contra todos md2 → disputa de posições md3 pela tabela, talvez em outro dia. Do usuário (06 e 07/11):

- **Início manual de cada fase**: "Iniciar fase N" depois que a anterior terminou (todas as séries decididas e nenhum empate do fim esperando). Entre as fases o torneio continua em andamento; encerra quando a última termina. A fase 1 nasce no "Iniciar" do torneio, como antes.
- **Check-in opcional por fase**, ligado na configuração dela, **sem data própria**: a organização o abre pelo botão (com a fase anterior terminada) e ele fecha quando a fase começa. Motivo: permitir pausa ou outro dia entre as fases sem marcar hora no site.
- **Corte**: a fase recebe todos ou os N melhores da anterior.
- **Quem entra**: os que jogaram a fase anterior, **sem os desistentes** ("não tem por que ocupar vaga para não jogar") e, com check-in, só os que confirmaram, na ordem da classificação da anterior, até o corte. O check-in abre para todos que seguem no torneio: quem está abaixo do corte é **suplente** e sobe se alguém de cima não confirmar. Sem check-in, o próximo da tabela entra direto.
- **Ordem só pela tabela**, sem ajuste manual: para mudá-la, corrige-se a fase anterior.
- **Correção da fase anterior depois do início da seguinte**: passa se não muda quem entra nem a ordem (frags, placar que não mexe na tabela); se muda, é recusada, e o caminho é desfazer o início da fase seguinte, possível só enquanto nada foi jogado nela. Escolhido no lugar de bloquear tudo (nem o rating de um mapa errado se corrigiria) e de corrigir sem refazer (como no suíço; a tabela da fase anterior ficaria contradizendo a seguinte).
- **Classificação final**: a última fase dá as primeiras posições; depois, de trás para a frente, cada fase anterior: primeiro os cortados nela, na ordem dela, e por último quem não confirmou presença ou desistiu. Quem estava disponível fica à frente de quem saiu. Com três fases (assumido), quem chegou à fase 2 e faltou à 3 fica à frente dos cortados da fase 1: foi mais longe.
- **Configuração muda até a fase começar** (formato, série, corte, desempate, check-in): na pausa dá para decidir se a fase 2 é md3 ou md5. Adicionar fase fecha no início do torneio; remover as que não começaram pode, o que também encerra o torneio pela fase atual se faltar gente.
- **Não repetição de campeões isolada por fase**: Desligada, Na série ou Na fase; com uma fase só, a mesma opção aparece como "No torneio" (ver `veto.md`).
- **Mapas e campeões por fase** (incremento depois deste, já decidido): Mapas e Campeões ganham "Mesmas regras em todas as fases" (marcado) e, desmarcado, uma aba por fase; Resultados fica do torneio. Até lá, valem as do torneio em todas as fases.

Assumido:

- Até 4 fases, com o nome fixo "Fase N". **Eliminação só como última fase**: os eliminados saem em faixas empatadas (3º–4º, 5º–8º) e não há onde cortar. O corte fica na fase que recebe, de 2 ao teto do formato dela; com menos candidatos que o corte, entram todos; com menos de 2, a fase não começa.
- No TDM, qualquer jogador do time confirma a presença do time; a organização confirma ou desfaz por qualquer um.
- Dentro da fase vale o seed da fase (lado `a`, quem começa o veto, a ordem da chave no desempate); a fase 1 continua com `teams.seed`. A contestação tardia da fase anterior fecha quando a seguinte começa (a regra de sempre: fecha quando a próxima série de um dos lados começa).
- Partida da fase 2 em diante com o prefixo da fase no código (`s2-pl-1-1`); a fase 1 continua sem prefixo e os links antigos valem.
- Avisos: check-in da fase aberto (jogadores dos candidatos, também no Discord), fase iniciada (Discord), partida pronta das séries da fase nova; o pódio só no fim, pela classificação final. Cortado, ausente e desistente só ganham "Terminou em Nº" quando a fase seguinte começa (antes, um suplente ainda pode subir).
- Desfazer o início do torneio continua apagando tudo, de todas as fases.
- No Discord, a série da fase 2 em diante sai como `1.2 (fase 2)`. Num torneio de várias fases, `teams.rating_at_start` fica gravado desde o início (uma fase por começar ainda pode ligar o desempate por menor rating).
- **Banco** (`20261218000000_stages`): `tournament_stages` ganha `advance`, `check_in`, `started_at`, `check_in_opened_at` e `finished_at` (preenchidos nos torneios já começados e encerrados); a trava passa a ser por fase, e o banco confere no commit a sequência 1..N, a eliminação por último e que uma fase só começa com a anterior terminada. `stage_teams` guarda o check-in e o seed de cada time nas fases 2 em diante (seed 1..K, congelado; só quem jogou a anterior; check-in só com ele aberto). Elegível = não desistiu antes do início da fase (o time que saiu inteiro da comunidade já conta como desistente).
- **API** (`stages/`): `GET .../stages/{n}` (quem entra, suplentes, check-ins, se dá para iniciar e por quê), `POST .../stages/{n}/check-in/open`, `POST`/`DELETE .../stages/{n}/teams/{team}/check-in`, `POST`/`DELETE .../stages/{n}/start`; `GET .../bracket?stage=N` (sem `stage`, a atual) com `standings_final`; "Seus torneios" com `awaiting_stage`, `stage_check_in_pending` e `substitute`; `/me/organizing` com `stage_due`, na mesma consulta dos empates.
- Remover as fases que faltam com a atual terminada encerra o torneio no PATCH, com o pódio no Discord pela classificação final: por isso o PATCH do torneio é registrado pelo `lifecycle`, que pode chamar `matches` (mesmo caminho e `operationId`; GET e DELETE continuam em `tournaments`).
- Desfazer o início de uma fase exige nenhuma atividade nela: veto, mapa enviado (mesmo anulado depois), resultado, evento de série ou screenshot, mesmo não anexado.

## Disputa de posições

Para a segunda etapa de um torneio em duas: depois de um todos contra todos (em md2, por exemplo), o 1º enfrenta o 2º, o 3º enfrenta o 4º, e assim por diante. Como fase seguinte, a ordem da chave é a classificação da anterior (ver "Fases ligadas"); como fase única, a da inscrição.

Do usuário (06/11):

- **Formato de uma fase** (`placement`), ao lado das eliminações, do todos contra todos e do suíço: "Disputa de posições", "Disputa de posiciones", "Placement matches". Partida `pl-1-<par>`.
- **Pares pela ordem da chave**: seed 1 x 2, 3 x 4..., todas as séries prontas no início, numa rodada só; o lado `a` é o seed melhor.
- **Colocação**: no par k, quem vence fica em 2k − 1 e quem perde em 2k, sem empate.
- **Série**: melhor de N ímpar, uma só para todas (precisa de vencedor); sem série par, sem desempate e sem final separada.

Assumido:

- **N ímpar**: o último seed não tem par e fica na posição dele, sem jogar (partida `Bye`, a última, como a folga do round robin; na chave, "P5 fica em 5º, sem adversário").
- **Limites**: de 2 a 128 participantes, como o suíço; orçamento de campeões (`worst_case_games`) = o melhor de N, porque cada um joga no máximo uma série.
- **O resto como nos outros formatos**: W.O. e desistência (o time que desiste perde por W.O. a série dele, pelo W.O. automático da API), veto e draft, contestação, rating por mapa. Todos os pares recebem `match.ready` no início, como uma rodada do suíço.
- **Colocação a cada par decidido**, como os eliminados da eliminação: "Seus torneios" já mostra "Terminou em 3º" para quem decidiu o par enquanto os outros jogam, e o sem par tem a posição desde o início. A aba Classificação, como na eliminação, só aparece no fim, com a série de cada um (vitória ou derrota, com o placar) ou "Sem adversário".
- **Tela**: a lista dos pares com os cartões da chave, e o código do cartão é o que está em jogo ("Disputa do 3º lugar"), também no título da série e nas listas que citam a série. No Discord fica `1.2`, como no round robin: "o veto da série Disputa do 3º lugar" não cabe nas frases de lá.
- **Banco**: só `best_of` (os outros campos da fase nulos, `tournament_stages_shape`), na migration `20261212000000_placement`.

## TDM

Times de 2 a 4 jogadores. Do usuário: times montados pelo organizador (sorteio aleatório com seed, balanceado pelo rating ou à mão), capitão definido pelo organizador ou pelo sorteio, quem sobra fica de reserva (por padrão, os últimos inscritos), substituição pelo organizador durante o torneio (entre mapas), quem sai da comunidade deixa o time desfalcado (W.O. só pela desistência ou sem ninguém), o mesmo campeão para dois do mesmo time na mesma partida configurável (padrão não), pool de mapas e veto/votação/sorteio como no duelo (feitos pelo time), e o TDM conta no ranking único. O resto é assumido:

- **Modo e tamanho** (`tournaments.mode` `duel` | `tdm`, `team_size` 1 no duelo e 2 a 4 no TDM): mudam até o torneio começar. Trocar com inscritos refaz os times no banco, na mesma transação (trigger `tournaments_mode_teams`); a tela avisa antes.
- **Vagas contam jogadores** (a inscrição é individual); mínimo de dois times (`2 × tamanho`, `tournament.max_participants_invalid {min}`). A checagem de campeões e o teto do formato usam ⌊vagas/tamanho⌋ times; o erro do teto sai em jogadores. O `engine` (`TournamentSetup`) recebe as vagas em jogadores (`format.not_enough_players {min, got}`). Ao iniciar, a checagem usa os times completos que entram, e os lados nunca passam de ⌊vagas/tamanho⌋.
- **Formação** (`/{tenant}/{torneio}/teams`, organização, antes do início): `GET .../teams`, `POST .../teams/draw {method: random | balanced}`, `POST .../teams`, `PUT`/`DELETE .../teams/{team}/members/{user}` (`team.full`; ir para a reserva). Elegíveis: quem fez check-in. O sorteio refaz todos os times; os últimos inscritos que não completam um time ficam de reserva, fora do equilíbrio; a seed fica em `tournaments.teams_seed`.
  - *Aleatório*: jogadores embaralhados pela seed (o mesmo ChaCha8 do seeding, com propósito próprio: o byte 8 vale 1 nos times e 0 no seeding, para a mesma seed nunca dar a mesma ordem nos dois) e cortados em times; capitão é o primeiro.
  - *Balanceado* ("minimizar a disparidade total", não só zigue-zague): minimiza a variância das médias de rating dos times (com times do mesmo tamanho, a soma dos quadrados das somas). Parte do zigue-zague pelo rating da comunidade (o inicial para quem não tem), troca pares entre times, sempre a troca que mais reduz a variância, até nenhuma reduzir, e recomeça 8 vezes de divisões embaralhadas pela seed, ficando com a melhor (empate: a de menor diferença maior-menor; o resto, a primeira). Capitão o de maior rating. Nunca é pior que o zigue-zague (teste de propriedade). O `engine` recusa tamanho 0 e jogador repetido (`FormationError`). Fica em `lifecycle` (lê o rating).
  - Tela: arrastar entre times, reserva e "Novo time" no computador; no celular e no teclado, o menu ⋯ de cada jogador.
- **Iniciar** exige pelo menos dois times, todos completos e com check-in de todos (`tournament.teams_incomplete {teams}`); reservas ficam fora. Quem desfaz o check-in depois da formação deixa o time incompleto; quem faz depois entra como reserva. Os times se refazem até o início; depois, só nome, capitão e substituição.
- **Ações do time**: quem faz o que é do lado (veto de mapas, voto, bans e escolhas de campeão, pedir e responder o desfazer, enviar, confirmar, contestar) é a regra do torneio, capitão ou qualquer um, e marca "Pronto" quem tem o que fazer antes das partidas (`veto.md`, "Pronto" por jogador); vale o primeiro clique e o histórico grava quem fez (a tela mostra "Time (jogador)"). Fora da regra, 403 `match.not_captain`. Anexar screenshot é de qualquer jogador. O **capitão** dá também o nome do time ("Time de {capitão}") e é o contato da organização; o que sai do time (formação, cancelar inscrição, substituição, sair da comunidade) passa a capitania ao jogador inscrito há mais tempo (trigger `participants_captain_handover` na inscrição cancelada). O organizador troca o capitão quando quiser (`PATCH .../teams/{team}`).
- **Nome do time**: opcional; o organizador e o capitão dão, até o torneio encerrar; 1 a 40 caracteres, único no torneio sem diferenciar maiúsculas (`team.name_taken`).
- **Substituição** (`POST .../teams/{team}/substitution {out, in}`, organizer, torneio em andamento): quem sai fica inscrito, sem time (reserva); quem entra é reserva ou outro membro ativo, já inscrito no time (a inscrição depois do início só existe assim, sem check-in, e não conta nas vagas). Time que desistiu não troca. Na aba Inscritos, no ⋯ de cada time.
- **Escalação**: a substituição vale a partir do próximo mapa enviado. **Desfalque**: quem sai da comunidade continua no time como histórico, sai do elenco e da escalação; o time só toma W.O. se desistir ou sem ninguém na comunidade.
- **Picks de campeão por jogador** (`engine::veto`, `team.rs`): depois dos bans da série, cada jogador do elenco escolhe o próprio campeão de **cada partida da série**, na ordem de `pick_order`, alternada (padrão) ou às cegas numa janela (`veto.md`, "Ordem das escolhas"; a janela aberta, com todos vendo tudo, só em série antiga). Cada escolha é definitiva (`veto.already_picked`) e diz a partida (`pick_game`). Sem repetir no time (padrão), as opções excluem o que um colega escolheu para a mesma partida; sem espelho, o que um adversário escolheu (quem confirma primeiro fica, `veto.champion_taken`). A janela (`team_pick_seconds`, padrão 60 s, de 15 a 300, no grupo Campeões recolhido) fecha quando todos escolheram tudo; no fim do prazo, cada escolha que falta é sorteada pela seed da série e pela posição no histórico, respeitando as regras. Quem entra no elenco depois (substituição) passa a faltar nas partidas ainda abertas e os picks reabrem para ele na partida em andamento; partida enviada ou série encerrada nunca reabre; quem entra antes do envio da partida 1 é sorteado com o resto quando ela é enviada (o draft nunca fica pela metade). A API informa a cada replay o que o histórico não diz (`Rosters { players, lineups, finished }`); sem o elenco, a vez é explícita (`Turn::RostersUnknown`) e a escolha ao vivo é recusada. O desfazer do organizador tira a última escolha (às cegas, não se desfaz); o pedido de desfazer é entre os times. Aviso `match.team_pick_open` (toast e Web Push, não Discord) quando os picks abrem. Nos times, `Event::TeamPick.player` é o jogador-alvo, gravado à parte de quem agiu (`actor`).
- **Regra de não repetição no TDM**: "quem" (cada jogador | o time) no grupo Campeões, padrão o jogador. Pela do jogador, sai o que ele escolheu para outra partida da série e, no escopo fase/torneio, o que jogou em outras séries, por qualquer time; pela do time, o que qualquer um do time escolheu ou jogou. Se a regra não deixa nenhum, ela cede; se nem assim sobra, cedem também as da partida (colegas e espelho). Cada jogador cede sozinho, na hora de escolher (o veto nunca trava): com o pool no limite, quem escolhe primeiro pode pegar o último campeão livre e o seguinte fica com a regra cedida; o resultado depende da ordem (revisão de regras: manter e avisar). O pick grava que cedeu (`gave_way`) e a tela mostra, por partida, o que cedeu e o motivo de cada campeão fora (banido, colega, adversário, repetição).
  - **Pick que conta**: numa partida enviada, o de quem a jogou; numa aberta, o de quem está no elenco agora, e só se ainda vale: quem saiu e voltou perde o pick antigo se, enquanto esteve fora, outro escolheu sem ceder algo que não caberia com ele, e passa a faltar. O substituto pode pegar o campeão de quem saiu antes de jogar (essa escolha não vai ser jogada; continua no histórico). Na série encerrada, a partida não jogada não conta.
- **Rating**: o mapa conta para a escalação de cada lado (ver `ratings.md`); o substituído não ganha nem perde pelos mapas depois da troca.
- **Avisos**: partida pronta para todos do time; o que pede ação, para quem pode agir (`veto.md`, "Quem age pelo time"); no Discord, os jogadores de cada time vão depois do nome do time, com a menção de quem vinculou o Discord.
