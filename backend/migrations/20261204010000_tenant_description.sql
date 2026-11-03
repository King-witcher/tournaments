-- Descrição da comunidade (03/11): os membros leem na aba Sobre, e quem recebe
-- um convite vê antes de entrar. Texto simples, um idioma só, com o limite da
-- descrição do torneio. Sem descrição é nulo: string vazia não é outro jeito
-- de dizer o mesmo.
alter table tenants
    add column description text
        constraint tenants_description
        check (description <> '' and length(description) <= 2000);
comment on column tenants.description is
    'Descrição em texto simples, em um idioma só; nulo: sem descrição.';
