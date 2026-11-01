-- Lembrete na véspera da exclusão (01/11): o email para a conta (se tem
-- email) e o aviso para o owner da comunidade saem uma vez, a 24 h da data.
-- `deletion_reminded_at` diz quando saiu e só existe com a exclusão agendada:
-- cancelar (e anonimizar a conta) o limpa junto, e o próximo agendamento
-- lembra de novo. Nenhum dado existente muda: as colunas nascem vazias.

alter table users
    add column deletion_reminded_at timestamptz,
    add constraint users_deletion_reminded_scheduled
        check (deletion_reminded_at is null or deletion_due_at is not null);

comment on column users.deletion_reminded_at is
    'Quando saiu o lembrete da exclusão agendada; nulo: ainda não saiu (ou não há exclusão).';

alter table tenants
    add column deletion_reminded_at timestamptz,
    add constraint tenants_deletion_reminded_scheduled
        check (deletion_reminded_at is null or deletion_due_at is not null);

comment on column tenants.deletion_reminded_at is
    'Quando o owner recebeu o aviso da exclusão agendada; nulo: ainda não recebeu (ou não há exclusão).';
