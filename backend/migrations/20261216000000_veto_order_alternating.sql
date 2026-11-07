-- Padrões do torneio novo (do usuário, revisão de 06/11).
--
-- A ordem do veto de mapas volta a ser o alternado por padrão. Só o default
-- muda: os torneios que existem ficam com a ordem que têm (a dos que já
-- começaram está congelada pelo trigger tournaments_veto_frozen, e o replay
-- deles lê essa coluna).
alter table tournaments alter column veto_order set default 'alternating';

-- O screenshot obrigatório também passa a nascer ligado, mas só com o bucket
-- configurado, o que o banco não sabe: a API sempre grava o valor, e o default
-- da coluna (false) fica como está.
