# Excluir conta, comunidade e torneio

Excluir torneio é imediato (seção no fim). Conta e comunidade, do usuário: as duas são agendadas para 7 dias (a comunidade sem histórico sai na hora, ver abaixo); a conta é anonimizada (não apagada), entrar de novo no prazo cancela, e quem é owner não pede (transfere a posse ou exclui a comunidade antes); a comunidade fica só leitura até a data, e só o owner pede, digitando o slug, e cancela. O "(assumido)" é decisão minha, fácil de mudar. Migration `20261114000000_account_and_tenant_deletion`.

## Conta

- `POST /me/deletion` grava `users.deletion_due_at` (agora + 7 dias). Conta com senha manda a atual (assumido). Owner: 409 `auth.owns_tenants` (regra do banco, `users_deletion_owner`, com a linha do usuário travada); no outro sentido ninguém vira owner com a exclusão agendada (na transferência, 409 `member.deletion_scheduled`).
- Ao agendar (assumido): todas as sessões são encerradas, também a do pedido, e o email da conta recebe a data em UTC e que entrar cancela (dentro dos limites de envio). A tela vai para a página de entrar com a data.
- Na véspera (24 h antes; do usuário, 01/11), o agendador manda um lembrete ao email da conta, com a data e que entrar cancela. Sai uma vez (`users.deletion_reminded_at`, que só existe com a exclusão agendada e some ao cancelar), pelo teto do dia (`reserve_notice`); conta sem email não recebe nada. Se a API estava parada e a data já passou, não lembra: exclui.
- Entrar de novo cancela (senha, Discord, Google ou a senha nova pelo link). O login traz `deletion_cancelled`; Discord e Google levam a `/account?account_deletion=cancelled`.
- Na data o agendador executa, uma conta por transação (o erro de uma fica no log): a conta sai de cada comunidade como quem sai sozinho (`members::depart`: W.O. automático nos torneios em andamento, sai das inscrições dos não iniciados); o nome exibido e os apelidos em todas as comunidades viram vazio, o **marcador de conta excluída** (o front mostra "Conta excluída"; a API devolve o marcador, nunca o texto); saem avatar, email, logins, inscrições de push e as sessões que sobraram. A linha de `users` fica, com `deleted_at`, e os torneios jogados continuam inteiros, com a marca "saiu".
- Email, Discord e Google ficam livres: entrar pelo mesmo Discord depois cria conta nova (assumido). Sessão de conta excluída não vale (`CurrentUser` ignora).
- No banco: nome vazio só (e sempre) na conta excluída, que não tem email, avatar nem exclusão agendada; no commit, não tem login, push nem participação ativa e todos os apelidos dela estão vazios; depois, não ganha login nem push e não volta.

## Comunidade

- **Sem histórico, na hora** (do usuário, 09/11): sem torneio em andamento ou encerrado (`tournaments.started_at`, que desfazer o início limpa: um início desfeito não conta), `POST /tenants/{slug}/deletion` apaga na hora, pela mesma confirmação do slug, com o `access_lost` e a cascata do agendador (imagens e screenshots pelas filas de sempre). O servidor decide, com os torneios e o tenant travados nessa ordem (a do início de um torneio, para os dois não se travarem), e a resposta diz o que fez: `outcome` `deleted` ou `scheduled` (com a data). `GET` na mesma rota diz antes o que faria (`immediate`, `scheduled`, `blocked`), para o Modal dizer a consequência real e o botão travar com torneio em andamento. Saiu na hora, o dono vai para o Início.
- Com histórico, o `POST` (owner, `confirm_slug`) grava `tenants.deletion_due_at` (agora + 7 dias) e `deletion_requested_by` (sempre o owner atual); `DELETE` cancela.
- **Só leitura**: o extractor `TenantMember` recusa todo método que não é de leitura com 409 `tenant.deletion_scheduled`, o que cobre toda rota de escrita sem mexer nelas; passam só cancelar e sair da comunidade (`DuringDeletion`). Fora do extractor (assumido): a prévia e o aceite de convites respondem o mesmo código; o agendador não abre o check-in nem avisa a hora de iniciar, e o que venceu nesse meio-tempo acontece na primeira passada depois de cancelar.
- Recusada com torneio em andamento (409 `tenant.tournament_in_progress`; assumido): só leitura congelaria a partida no meio. O banco confere nos dois sentidos (`tenants_deletion_in_progress`, `tournaments_tenant_deleting`).
- Os membros veem o aviso com a data no topo (o owner, com o caminho para cancelar nas configurações, onde a transferência de posse some até lá); as páginas recarregam ao agendar e cancelar. Os membros não recebem aviso por push nem email (do usuário, 01/11). O owner recebe, na véspera (24 h antes), o aviso `tenant.deletion_due` (sininho, toast e Web Push, que leva às configurações), uma vez (`tenants.deletion_reminded_at`, limpo ao cancelar).
- A página de convite de comunidade em exclusão diz que ela não aceita mais membros, não o texto da escrita recusada.
- Na data o agendador apaga o tenant, um por transação, e a cascata leva tudo; antes, cada membro ativo recebe `access_lost`. Os screenshots no bucket saem pela fila (ver `screenshots.md`).

## Torneio

Do usuário, 03/11. `DELETE /tenants/{slug}/tournaments/{t}`, **na hora**, no ⋯ do torneio.

- **Quem**: organizer e owner enquanto não começou (rascunho, inscrição, check-in); depois de começar, inclusive encerrado, só o owner (403 `tenant.forbidden`). Com a comunidade em exclusão agendada, a rota recusa como toda escrita.
- **Confirmação**: no rascunho, o Modal basta. Fora dele, `confirm_slug` digitado no corpo (422 `tournament.confirmation_mismatch`).
- **Não avisa ninguém**: nem sininho, nem push, nem Discord. As páginas abertas só recebem a recarga de sempre (`Event::Tournament`, `Event::Tenant`, `Event::Me`) e caem no "não encontrado"; a home da organização também (mapa contestado, hora de iniciar).
- **O que some junto**: a cascata leva séries, mapas, veto, draft, escalações, times e inscritos, e o link de inscrição (a coluna dele) deixa de valer (nenhum trigger de recusa segura: eles reconhecem o torneio já apagado). O trigger `tournaments_path_forgotten` apaga as notificações que levam a ele e os avisos pendentes na fila do Discord e no outbox; quem perdeu uma notificação não vista recebe a contagem nova (`Event::Unread`). O endereço fica livre.
- **Ranking**: apagar um mapa confirmado já marca o tenant (`rating_dirty`), e o agendador ou a próxima leitura refazem do zero, sem os mapas dele; em até 10 s, com a recarga das páginas abertas.
- **Arquivos**: capa e uploads pendentes entram em `image_deletions`, screenshots em `screenshot_deletions`, e a rotina diária os apaga.
- **Agendador**: `confirm_all_due` e o relógio do veto procuram o torneio de novo antes de agir e seguem sem ele.
