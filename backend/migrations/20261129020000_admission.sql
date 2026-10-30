-- Inscrição fechada no lugar do torneio privado, e inscrição aberta até o
-- início (decisões em "Torneios" no docs/decisions.md).

-- 1. Todo torneio publicado aparece para todos os membros da comunidade. O
-- que a opção controla agora é quem pode se inscrever pelo botão: 'open' (o
-- membro se inscreve) ou 'invite_only' (só pelo link de inscrição ou pela
-- organização). Os privados de antes viram 'invite_only', que é o que eles
-- já eram para quem queria entrar.
alter table tournaments
    add column admission text not null default 'open'
        check (admission in ('open', 'invite_only'));
update tournaments set admission = 'invite_only' where visibility = 'private';
alter table tournaments drop column visibility;

-- 2. Reabrir a inscrição durante o check-in deixou de existir (a inscrição
-- fica aberta até o início), e com ela o marco que suspendia a abertura
-- automática. Quem desliga o check-in volta para a inscrição, como antes.
alter table tournaments drop column registration_reopened_at;

-- 3. Quem se inscreve durante o check-in já entra com o check-in feito (pelo
-- botão, pelo link ou pela organização). Vale na entrada: desfazer o check-in
-- depois continua permitido. Fora do check-in, nada muda.
create function participants_check_in_on_entry() returns trigger
language plpgsql as $$
begin
    if new.checked_in_at is null
       and (select status from tournaments where id = new.tournament_id) = 'check_in'
    then
        raise exception 'inscrição durante o check-in entra com check-in (torneio %)', new.tournament_id
            using errcode = 'check_violation', constraint = 'participants_check_in_on_entry';
    end if;
    return new;
end $$;

create trigger participants_check_in_on_entry
    before insert on participants
    for each row execute function participants_check_in_on_entry();
