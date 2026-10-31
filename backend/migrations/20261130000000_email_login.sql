-- O login com senha passa a ser pelo email da conta ("Logins" em
-- docs/decisions/auth.md): o username sai, e password_credentials fica só com
-- o hash. O email já era único, confirmado e obrigatório na conta com senha
-- (password_credentials_email_required), e o único dele serve de índice ao
-- login.
--
-- Dados existentes: toda conta com senha já tem email, então ninguém perde o
-- acesso. O nome exibido, que nasceu do username no cadastro, fica como está.
drop index password_credentials_username_idx;
alter table password_credentials drop column username;
