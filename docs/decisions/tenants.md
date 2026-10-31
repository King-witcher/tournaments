# Comunidades (tenants), membros e convites

Excluir a comunidade: `deletion.md`.

- Um banco só, com `tenant_id` em tudo o que pertence a um tenant. Toda função de acesso a dados de tenant recebe um `TenantId` tipado (esquecer o filtro vira erro de compilação). RLS pode entrar depois como segunda camada.
- Tenant na URL por caminho: `/{locale}/{tenant}/...`. Papéis em `tenant_members`: owner, organizer, player. Na interface, tenant é **comunidade** (es: comunidad).
- Cada usuário é owner de no máximo **3** tenants (`MAX_OWNED_TENANTS`, fixo no código); ser organizer ou player em outros não conta.
- **Slug**: o front sugere a partir do nome, o usuário ajusta na criação, e depois **não muda** (links nunca quebram). a-z, 0-9 e hífen simples, 3 a 32 caracteres, com palavras reservadas para rotas do front (`new`, `invite`...).
- Um owner por tenant (quem criou, até transferir). O banco garante exatamente um.
- Página do tenant **só para membros**: para quem é de fora, a API responde 404 `tenant.not_found`, igual a tenant inexistente, para não revelar que existe. A prévia do convite (`GET /invites/{token}`, pública) é exceção: ter o token já é o convite.
- **Perfil por tenant**: só o **apelido** (`tenant_members.nickname`), que começa com o nome exibido e o membro edita. **Pode repetir** dentro do tenant: as telas mostram o avatar ao lado e as rotas identificam por `user_id`.
- **Configurações**: os parâmetros do Elo, pelo owner (ver `ratings.md`). Ranking ligado/desligado e outro sistema (Glicko-2) ficam para quando houver o que escolher.

## Convites

- Link **multiuso**, vale 7 dias, revogável. Owner e organizer geram; quem entra vira **player**. Só o hash do token fica no banco (o link aparece uma vez, na criação). Vai sem idioma (`/invite/{token}`); o proxy do next-intl manda cada pessoa para o seu. O link de inscrição de um torneio (ver `tournaments.md`) também faz entrar no tenant.
- Aceitar sendo já membro não muda papel nem apelido. A prévia traz o slug da comunidade, o início do torneio (link de inscrição) e, para quem está logado, se já é membro, se saiu (e o apelido de antes) e se já está inscrito: a tela troca o botão por "você já é membro" com o link, "Inscrever-se" (membro no link de inscrição) ou diz que volta como jogador com apelido e histórico (quem saiu).
- **Quem vai entrar vê o que importa para decidir**: a prévia traz `owner_name` (apelido do dono *de agora*: a comunidade não grava quem a criou) e `member_count` (ativos); a página não mostra a validade, que é informação de quem gerencia (a tabela da aba Convites continua com ela; `expires_at` segue na API).

## Membros

- O owner alterna player ↔ organizer, remove, transfere a posse e exclui a comunidade; o organizer remove players; cada um sai sozinho.
- **Sair ou ser removido deixa o membro inativo, com o histórico** (`tenant_members.left_at`, e `removed_by` se alguém removeu): a linha não é apagada (o banco nem deixaria, se ele jogou torneio iniciado). O inativo perde o acesso (404 `tenant.not_found`), some da lista e de "Minhas comunidades" e não pode ser inscrito. Sai das inscrições dos torneios que não começaram, desiste dos em andamento em que está na chave (W.O. nas pendentes) e, nos encerrados, fica como está. Os convites que criou são revogados. Nas chaves e listas de torneios iniciados o nome continua, com a marca "saiu".
- **Quem remove**: o owner remove qualquer um (menos ele mesmo); o organizer só players (403 `tenant.forbidden`). Ninguém remove o owner (409 `member.is_owner`); remover a si mesmo é 409 `member.remove_self` (sair é outra rota). O owner não sai: transfere a posse antes (`member.owner_must_transfer`). Rotas: `DELETE /tenants/{slug}/members/{user_id}` e `DELETE /tenants/{slug}/me`.
- **Remover não é banimento** (assumido): quem foi removido volta por qualquer convite válido; para impedir, revogue os convites.
- **Voltar por convite reativa a mesma linha** (assumido): apelido e histórico de antes, **como player** (também quem era organizer); `joined_at` continua o da primeira entrada e `invite_id` passa a ser o da volta. Vale para o link de inscrição também.
- **Desfazer o início** de um torneio tira as inscrições de quem saiu da comunidade ou desistiu (assumido).
- **Transferir a posse** (`POST /tenants/{slug}/transfer-ownership` com `user_id` e `confirm_slug`, 422 `tenant.confirmation_mismatch`): o owner escolhe um membro ativo. O novo owner respeita o limite de 3 (409 `member.owner_limit_reached {max}`, contado com a linha do usuário travada). O antigo vira **organizer**. Numa transação, rebaixa o atual antes de promover o novo (a ordem inversa bate no índice único).
