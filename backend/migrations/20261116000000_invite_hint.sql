-- Os 4 primeiros caracteres do token do convite, em claro, só para a
-- organização reconhecer o link na lista (e saber por qual entrou quem
-- entrou). O segredo continua sendo o resto, guardado só como hash: sobram
-- ~105 bits. Nulo nos links criados antes desta coluna.
alter table tenant_invites
    add column token_hint text,
    add constraint tenant_invites_token_hint_format
        check (token_hint is null or token_hint ~ '^[1-9A-HJ-NP-Za-km-z]{4}$');
