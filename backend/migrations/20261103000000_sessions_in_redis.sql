-- Sessões e erros de senha passam para o Redis ("Redis" em docs/decisions.md).
-- As sessões abertas não migram: quem estava logado entra de novo. Os erros de
-- senha valiam por 15 minutos.
--
-- A limpeza dos erros quando o username muda ou o login sai (trigger da
-- migration 20261030000000_delete_rules) não tem mais o que limpar: pela API,
-- trocar ou remover o username exige a senha atual, e o acerto já zera os
-- erros; no resto, eles expiram com a janela.
drop trigger user_identities_clear_failures on user_identities;
drop function user_identities_clear_failures();

drop table password_failures;
drop table sessions;
