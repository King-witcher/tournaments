-- Login com o Google ("Autenticação" em docs/decisions.md): mais um provedor
-- externo em user_identities. O provider_user_id é o `sub` do OpenID Connect,
-- que o Google descreve como até 255 caracteres ASCII, sem reaproveitar.
alter table user_identities
    drop constraint user_identities_provider_check,
    add constraint user_identities_provider_check check (provider in ('discord', 'google')),
    add constraint user_identities_google_sub_format
        check (provider <> 'google' or provider_user_id ~ '^[!-~]{1,255}$');
