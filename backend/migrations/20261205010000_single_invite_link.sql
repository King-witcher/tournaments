-- Um link de convite por comunidade e um de inscrição por torneio
-- (docs/decisions/tenants.md, "Convites"; decisão do usuário, 03/11): gerenciar
-- vários links e saber por onde cada um veio era confuso, e o link é público.
--
-- O link vira uma coluna `not null unique` da própria comunidade e do próprio
-- torneio, no lugar da tabela tenant_invites. Sem trigger nenhum, o banco
-- garante o que importa: no máximo um por escopo (é uma coluna), sempre um
-- (`not null`), nasce e some com o dono, e não há como ter revogado, validade
-- ou autor. Girar é trocar o valor: o antigo deixa de existir.
--
-- O token fica em claro para aparecer a qualquer hora, exceção à regra "só o
-- hash" (docs/decisions/stack.md, "Ids"): ele só dá entrada como jogador e se
-- troca a qualquer momento.
--
-- Dados existentes: os links de antes saem (produção sem uso, decisão do
-- usuário), e toda comunidade e todo torneio ganham um token novo.

-- 1. Sair da comunidade não olha mais os convites (nem há de quem sejam); a
-- inscrição em torneio que não começou continua impedindo.
create or replace function tenant_members_leave_guard() returns trigger
language plpgsql as $$
begin
    if exists (select 1 from participants
               where tenant_id = new.tenant_id and user_id = new.user_id
                 and not tournament_started(tournament_id))
    then
        raise exception 'membro % sai inscrito em torneio que não começou', new.user_id
            using errcode = 'check_violation', constraint = 'tenant_members_left_registered';
    end if;
    return new;
end $$;

-- 2. A atribuição (por qual link cada um entrou ou se inscreveu) e a tabela.
-- O trigger de autor ativo sai com a tabela; a função, em seguida.
alter table participants drop column invite_id;
alter table tenant_members drop column invite_id;
drop table tenant_invites;
drop function tenant_invites_creator_active();

-- 3. Gerador só para preencher as linhas de agora; depois é a API que gera, com
-- o mesmo formato (`token::random_token`: 22 caracteres base58, 128 bits).
-- Cada byte vem do `gen_random_uuid`, que usa o gerador forte do Postgres e não
-- pede extensão; os bytes 6 e 8 têm os bits fixos de versão e variante e ficam
-- de fora, e os acima de 231 (4 * 58) são descartados, para não haver viés.
create function new_invite_token() returns text
language plpgsql as $$
declare
    alphabet constant text :=
        '123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz';
    token text := '';
    random_bytes bytea;
    byte int;
begin
    while length(token) < 22 loop
        random_bytes := uuid_send(gen_random_uuid());
        for position in 0..15 loop
            continue when position in (6, 8);
            byte := get_byte(random_bytes, position);
            if byte < 232 and length(token) < 22 then
                token := token || substr(alphabet, byte % 58 + 1, 1);
            end if;
        end loop;
    end loop;
    return token;
end $$;

-- 4. O link de cada escopo. Sem `default`: quem cria a linha gera o token. A
-- busca pelo token (GET /invites/{token}) é uma consulta só, que passa pelos
-- dois índices únicos.
alter table tenants add column invite_token text;
update tenants set invite_token = new_invite_token();
alter table tenants
    alter column invite_token set not null,
    add constraint tenants_invite_token_key unique (invite_token),
    add constraint tenants_invite_token_format
        check (invite_token ~ '^[1-9A-HJ-NP-Za-km-z]{22}$');

alter table tournaments add column invite_token text;
update tournaments set invite_token = new_invite_token();
alter table tournaments
    alter column invite_token set not null,
    add constraint tournaments_invite_token_key unique (invite_token),
    add constraint tournaments_invite_token_format
        check (invite_token ~ '^[1-9A-HJ-NP-Za-km-z]{22}$');

drop function new_invite_token();
