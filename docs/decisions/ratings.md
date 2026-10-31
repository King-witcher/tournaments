# Rating

Trait `RatingSystem` no `engine` (`rating/`); Elo primeiro. Decidido pelo usuário, salvo o marcado "assumido" (fácil de mudar).

- **Derivado, nunca editado direto**: reprocessar do zero, pelo `engine` (`rating::replay`), em ordem determinística, os mapas confirmados dos torneios que contam e os ajustes manuais. Ordem: horário de confirmação do mapa (`confirmed_at`) ou do ajuste; no mesmo instante, mapas antes de ajustes; depois o id. `ratings` (rating, mapas, vitórias, derrotas, último mapa) e `rating_history` (± e rating depois de cada evento) são cache, apagável e refeito. Correção, anulação, desfazer o início e reabertura recalculam o tenant inteiro (milhares de mapas: milissegundos).
- **Um ranking por comunidade, para todos os modos** (o TDM conta no mesmo do duelo; sem coluna `modality`). O de FFA continua em aberto.
- **Elo por mapa**, inicial 1500: cada mapa confirmado de `match_games` é um jogo do rating, não a série; mapas sem nome nem frags (séries antigas) contam igual, pelo vencedor. **W.O. não conta**, nem do organizador nem automático (não tem mapa); mapas confirmados antes do W.O. contam (assumido).
- **K decai com os mapas já computados da pessoa**: K = k_min + (k_max − k_min)·e^(−n/k_decay), padrões 1500 (inicial), 32 (k_min), 80 (k_max) e 10 (decaimento): K = 32 + 48·e^(−n/10), 80 no primeiro mapa, ~50 no décimo, ~32 depois de ~40. Configurável por comunidade.
- **Times** (no duelo, é o Elo de dois jogadores): cada mapa entra com a **escalação** de cada lado (`match_game_players`). E do vencedor = 1/(1 + 10^((média_perdedor − média_vencedor)/400)), com a média aritmética (`f64`, sem arredondar; assumido) dos ratings de cada lado; cada vencedor ganha round(K_i·(1 − E)) e cada perdedor perde round(K_i·(1 − E)), com K_i o K do próprio jogador. No duelo os valores não mudaram. `rating::Event::Game` tem `winners` e `losers`; o `replay` recusa lado vazio (`EmptySide`) e jogador repetido (`RepeatedPlayer`).
- **Ratings inteiros** (assumido): cada variação é arredondada no evento (metade para longe do zero), para a soma dos ± exibidos bater com o rating. Com o mesmo K, o que um ganha o outro perde; novo contra veterano não é soma zero. Sem piso nem teto. `exp`/`powf` são os da plataforma: um arredondamento a ~1e-13 da metade poderia diferir entre Windows e Linux; um teste fixa valores para pegar isso.
- **Corrigir um mapa confirmado o move para o fim da ordem** (a correção grava `confirmed_at = now()`), então até mudar só os frags pode mexer 1 ou 2 pontos em quem jogou depois. Manter o horário original resolveria; fica para quando incomodar.

## Configuração

- Faixas: inicial 100–3000, K mínimo 1–100, K máximo 1–200 e não abaixo do mínimo (igual = K fixo), decaimento 1–100 mapas (repetidas como `check` na tabela). **Só o owner muda**, na aba Configurações; todos leem.
- **Fator K mostrado como o Elo padrão** (a vitória entre iguais vale metade do K): o usuário propôs mostrar o K dobrado, mas K é parâmetro conhecido e dobrar o tornaria diferente em toda outra ferramenta; o usuário aceitou. Rótulos "Rating inicial", "Fator K inicial", "Fator K mínimo", "Queda do K, em partidas".
- Como o rating é sempre refeito do zero, salvar sem recalcular não existe: o botão abre um Modal ("Cancelar", "Salvar e recalcular") que avisa que o histórico inteiro será recalculado.
- **Torneio "conta para o rating"**: padrão sim (também os já existentes); no grupo Formato, muda até começar (`tournament.setup_locked`).
- **Ajuste manual** (organizer ou owner): −1000 a 1000, sem zero, para membro ativo; motivo de 1 a 280 caracteres, autor e horário gravados; não conta como mapa (não muda o K). Não se apaga nem edita: corrige-se com outro ajuste. A lista com motivo e autor é só da organização; os outros veem a soma dos ajustes no resumo.

## Recálculo

- **Sem chamada nova no `matches`**: um trigger grava uma marca em `rating_dirty` quando muda um mapa confirmado (e a escalação dele), um ajuste, a configuração ou o "conta para o rating" (mapa pendente, contestado ou anulado sem ter confirmado não marca). A marca é uma linha nova, nunca update de uma linha por tenant: quem grava um mapa não espera ninguém. Recalculam o agendador (a cada 10 s, depois das confirmações por tempo, um tenant por transação), a leitura do ranking e do rating de um membro (se há marca) e as rotas de ajuste e configuração (na própria transação). O recálculo trava o tenant (advisory lock), apaga as marcas que vê e só depois lê o histórico; marca de transação ainda não confirmada fica e recalcula de novo. Se o cache sai igual, não grava nem emite; se muda, emite `tenants::changed`.
- **Custo no Redis**: cada mudança do ranking recarrega as páginas abertas da comunidade e dos torneios (uma leitura de sessão cada); o agendador junta as de 10 s, e um mapa confirmado costuma dar uma recarga pela partida e outra, até 10 s depois, pelo ranking. Se pesar, um evento próprio do ranking evita a segunda.
- **Cache sem FK para o mapa e o ajuste** (`rating_history.game_id`, `adjustment_id`): de propósito; a FK travaria o mapa contra o recálculo (anular um mapa enquanto o histórico é regravado dava deadlock). O que some marca o tenant.
- `rating_history` não guarda adversário; os últimos mapas da página da pessoa leem o lado dela na escalação e mostram o time do outro lado (`RatedGame.opponent`).
- Os ratings de quem já jogou aparecem na primeira passada do agendador depois do deploy (a migration marca todos os tenants).

## Quem vê

- Todos os membros veem o ranking e o rating de cada um. Página da pessoa: resumo (rating, posição, mapas, V–D, último mapa, soma dos ajustes) e os últimos 20 mapas que contaram (torneio, adversário, mapa, resultado, ±, rating depois), de qualquer torneio publicado.
- **Ranking**: só membros ativos com algum mapa ou ajuste; quem saiu some, mas os mapas dele continuam no rating dos outros. Ordem: rating, mais mapas, apelido; empatados dividem a posição (1, 2, 2, 4). Menos de 10 mapas = **provisório**.
