-- Email como foi escrito, para dev e staging ("Email" em docs/decisions.md):
-- com EMAIL_STRICT desligado, a API grava o endereço sem juntar as variações
-- (o "+etiqueta", os pontos do Gmail, googlemail.com), para várias contas de
-- teste usarem a mesma caixa. O check passa a aceitar as duas formas; a
-- canônica (EMAIL_STRICT ligado, o padrão e o de produção) continua sendo a
-- única que a API grava lá. O único continua sobre o que a API grava.
--
-- Dados existentes: o check novo só afrouxa o antigo (todo email que passava
-- passa), então nada muda.
alter table users drop constraint users_email_format;
alter table users
    add constraint users_email_format check (
        length(email) <= 254
        -- Parte local de letras, números, ".", "_" e "-", mais a "+etiqueta"
        -- opcional (também com "+"), e domínio com pelo menos um ponto. O
        -- resto do formato (ponto nas pontas, terminação do domínio) fica com
        -- a API, como antes.
        and email ~ '^[a-z0-9._-]+(\+[a-z0-9._+-]+)?@[a-z0-9-]+(\.[a-z0-9-]+)+$'
    );
