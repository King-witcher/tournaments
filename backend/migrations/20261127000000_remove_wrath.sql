-- Wrath não existe no Quake Champions; entrou por engano na lista inicial.
-- Sem uso em nenhum veto, a linha sai; se algum veto já a escolheu ou baniu,
-- ela fica com active = false para a história continuar coerente.
delete from champions c
where c.slug = 'wrath'
  and not exists (select 1 from match_veto_events e where e.champion_id = c.id);

update champions set active = false where slug = 'wrath';
