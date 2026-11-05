# Consistência do banco

O banco recusa estado inválido e entidade órfã mesmo que a API erre; a API continua conferindo antes, para devolver o código certo. Checklist para migration nova em `backend/CLAUDE.md`; testes por SQL direto em `tests/api/schema/`. As regras de cada domínio estão nas migrations (cada uma comenta o que cria).

## Exclusões

- **Excluir usuário** é recusado se for owner de algum tenant ou tiver participado de torneio iniciado (a chave antiga não perde um lado). Logins vão junto; as sessões no Redis ficam sem efeito até expirar. A autoria (quem inscreveu, marcou check-in, enviou, contestou, confirmou, agiu no histórico) fica nula. A "Excluir conta" da interface **anonimiza** em vez de apagar a linha (`deletion.md`).
- **Excluir tenant** apaga tudo dentro (pela API, agendado: `deletion.md`). **Excluir torneio** também, com resultados, mapas e histórico (quem pode é regra da API: `deletion.md`).
- **Inscrito, time e fase de torneio iniciado** não saem avulsos, só com o torneio ou o tenant; também ninguém entra e seed e check-in não mudam. Por isso o membro inscrito num torneio iniciado não sai do tenant: sair mantendo o histórico é ficar inativo, não linha apagada.

## Regras de estado

- **Membro inativo**: o owner nunca fica inativo; `removed_by` só em quem saiu, nunca ele mesmo, nulo se o autor for apagado; sair exige nenhuma inscrição em torneio não iniciado; inativo não é inscrito; torneio que volta a não ter começado (desfazer o início) não fica, no commit, com inscrito inativo.
- **Desistência e W.O. automático**: só com o torneio em andamento, só de quem tem seed, não se desfaz nem muda de data; torneio que volta a não ter começado não fica com desistente. Valem para o time (`teams_withdrawal_*`). O perdedor do W.O. automático é o time que desistiu ou sem ninguém na comunidade; o W.O. automático não tem placar, quem enviou nem quem confirmou, e o evento `auto_walkover` tem só o vencedor e nenhum autor.
- **Times**: no duelo, o time de um jogador nasce com a inscrição e some com ela; no commit, o capitão é do time, todo inscrito do duelo tem time e todo time do duelo tem exatamente um jogador e nenhum nome; a escalação de cada mapa é de jogadores do time daquele lado, não muda e tem alguém dos dois lados.
- **Owner**: exatamente um por tenant e membro. "No máximo um" vale na hora (índice único parcial); "pelo menos um", no commit.
- Quem inscreveu e quem marcou check-in são membros do mesmo tenant (FK composta).
- **Link de convite**: um por comunidade e um por torneio, sempre (`invite_token not null unique`, nas duas tabelas), em base58 de 22 caracteres.
- **Estado e forma**: status check-in só com janela; tradução nunca no idioma original; ao menos uma fase por torneio; vagas nunca abaixo dos inscritos; idioma original fixo; slug do torneio só muda no rascunho; regras de campeões e cada formato de fase com as suas colunas; senha só como hash argon2id, avatar só por https, nome exibido não vazio, campeão com nome único.

## Configuração em colunas

As regras de campeões (`tournaments`) e as fases (`tournament_stages`, em que `format` diz quais colunas valem) ficam em colunas, não em jsonb (05/11). Em JSON, um campo novo no `engine` compilava e passava nos testes, porque o banco de teste nasce vazio, e só os torneios já gravados davam 500. Em coluna, a tradução (`tournaments/db/columns.rs`) não compila sem ela e recriar um check confere as linhas gravadas. jsonb fica só no que é passageiro e varia por código: `notifications.notice` (30 dias), `outbox.event` e `discord_messages.params` (até 1 h).

Mudar o comportamento do `engine` ainda pode mudar um torneio gravado, que sai do replay; coluna não resolve isso, e o teste `engine/tests/replay_golden` avisa.

## Sala de veto

Lugares de acordo com o modo (`team_size` por lado, no commit) e pool de acordo com o modo do mapa; um token por lugar; validade obrigatória; nome de 1 a 32 caracteres, sem espaço nas pontas, definido uma vez; configuração que não muda (o replay lê dela); histórico que só cresce, na forma do `match_veto_events`, com os lugares da própria sala. Apagar a sala leva tudo; excluir a conta deixa a sala sem autor. Uma migration que apague um mapa ou campeão precisa apagar antes as salas que os usam.

## Fora do banco, de propósito

- Ordem entre horários de ações de transações diferentes (cada uma grava o próprio `now()`, e uma pode ter começado antes da outra) e datas agendadas no futuro; histórico imutável (os testes envelhecem essas datas direto no banco).
- O mesmo token numa comunidade e num torneio: 128 bits aleatórios não se repetem, e um trigger para isso não se paga (`tenants.md`, "Convites").
- O limite de 3 tenants por owner e o pool de campeões: regras da API, que mudam sem migration.
- O "pronto" do veto sem autor pode repetir: não conta para ninguém, e exigir o autor colidiria com o `set null` de quando a conta é apagada.
