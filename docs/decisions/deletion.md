# Excluir conta e excluir comunidade

Do usuário: as duas são agendadas para 7 dias; a conta é anonimizada (não apagada), entrar de novo no prazo cancela, e quem é owner não pede (transfere a posse ou exclui a comunidade antes); a comunidade fica só leitura até a data, e só o owner pede, digitando o slug, e cancela. O "(assumido)" é decisão minha, fácil de mudar. Migration `20261114000000_account_and_tenant_deletion`.

## Conta

- `POST /me/deletion` grava `users.deletion_due_at` (agora + 7 dias). Conta com senha manda a atual (assumido). Owner: 409 `auth.owns_tenants` (regra do banco, `users_deletion_owner`, com a linha do usuário travada); no outro sentido ninguém vira owner com a exclusão agendada (na transferência, 409 `member.deletion_scheduled`).
- Ao agendar (assumido): todas as sessões são encerradas, também a do pedido, e o email da conta recebe a data em UTC e que entrar cancela (dentro dos limites de envio). A tela vai para a página de entrar com a data.
- Entrar de novo cancela (senha, Discord, Google ou a senha nova pelo link). O login traz `deletion_cancelled`; Discord e Google levam a `/account?account_deletion=cancelled`.
- Na data o agendador executa, uma conta por transação (o erro de uma fica no log): a conta sai de cada comunidade como quem sai sozinho (`members::depart`: W.O. automático nos torneios em andamento, sai das inscrições dos não iniciados, convites revogados); o nome exibido e os apelidos em todas as comunidades viram vazio, o **marcador de conta excluída** (o front mostra "Conta excluída"; a API devolve o marcador, nunca o texto); saem avatar, email, logins, inscrições de push e as sessões que sobraram. A linha de `users` fica, com `deleted_at`, e os torneios jogados continuam inteiros, com a marca "saiu".
- Email, Discord e Google ficam livres: entrar pelo mesmo Discord depois cria conta nova (assumido). Sessão de conta excluída não vale (`CurrentUser` ignora).
- No banco: nome vazio só (e sempre) na conta excluída, que não tem email, avatar nem exclusão agendada; no commit, não tem login, push nem participação ativa e todos os apelidos dela estão vazios; depois, não ganha login nem push e não volta.

## Comunidade

- `POST /tenants/{slug}/deletion` (owner, `confirm_slug`) grava `tenants.deletion_due_at` (agora + 7 dias) e `deletion_requested_by` (sempre o owner atual); `DELETE` cancela.
- **Só leitura**: o extractor `TenantMember` recusa todo método que não é de leitura com 409 `tenant.deletion_scheduled`, o que cobre toda rota de escrita sem mexer nelas; passam só cancelar e sair da comunidade (`DuringDeletion`). Fora do extractor (assumido): a prévia e o aceite de convites respondem o mesmo código; o agendador não abre o check-in nem avisa a hora de iniciar, e o que venceu nesse meio-tempo acontece na primeira passada depois de cancelar.
- Recusada com torneio em andamento (409 `tenant.tournament_in_progress`; assumido): só leitura congelaria a partida no meio. O banco confere nos dois sentidos (`tenants_deletion_in_progress`, `tournaments_tenant_deleting`).
- Os membros veem o aviso com a data no topo (o owner, com o caminho para cancelar nas configurações, onde a transferência de posse some até lá); as páginas recarregam ao agendar e cancelar. Sem aviso por push nem email.
- Na data o agendador apaga o tenant, um por transação, e a cascata leva tudo; antes, cada membro ativo recebe `access_lost`. Os screenshots no bucket saem pela fila (ver `screenshots.md`).
