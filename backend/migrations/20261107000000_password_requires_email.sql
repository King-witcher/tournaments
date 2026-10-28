-- Conta com senha tem email ("Autenticação" em docs/decisions.md): é por ele
-- que a senha esquecida volta. Sem isso, quem esquecia a senha ficava preso,
-- porque pôr o email e remover a senha pedem a senha atual.

-- Dados antigos: as contas com senha e sem email (as de antes do email
-- obrigatório) eram de teste e saíram à mão, antes desta migration (em dev,
-- ganharam um email inventado). Sobrou alguma, a migration falha sem mudar
-- nada, em vez de apagar conta.
do $$
begin
    if exists (select 1 from password_credentials p
               join users u on u.id = p.user_id
               where u.email is null) then
        raise exception 'há contas com senha e sem email: ponha um email ou apague-as antes desta migration'
            using errcode = 'check_violation', constraint = 'password_credentials_email_required';
    end if;
end $$;

-- A senha só entra na conta que tem email, e o email só sai da que não tem
-- senha. O `for share` e o lock da linha no update do email põem as duas
-- mudanças da mesma conta em fila: a segunda já vê a primeira.
create function password_credentials_email_required() returns trigger
language plpgsql as $$
begin
    if (select email from users where id = new.user_id for share) is null then
        raise exception 'usuário % sem email não pode ter senha', new.user_id
            using errcode = 'check_violation', constraint = 'password_credentials_email_required';
    end if;
    return new;
end $$;

create trigger password_credentials_email_required
    before insert or update of user_id on password_credentials
    for each row execute function password_credentials_email_required();

create function users_email_required_by_password() returns trigger
language plpgsql as $$
begin
    if exists (select 1 from password_credentials where user_id = new.id) then
        raise exception 'usuário % tem senha e não pode ficar sem email', new.id
            using errcode = 'check_violation', constraint = 'password_credentials_email_required';
    end if;
    return new;
end $$;

create trigger users_email_required_by_password
    before update of email on users
    for each row when (new.email is null and old.email is not null)
    execute function users_email_required_by_password();
