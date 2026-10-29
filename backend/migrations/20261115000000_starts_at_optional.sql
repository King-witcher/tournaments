-- A data de início deixa de ser obrigatória fora do rascunho (decisão em
-- "Torneios" no docs/decisions.md): sem data, o check-in não abre sozinho e
-- não há aviso de hora de iniciar. Só remove uma regra; nenhuma linha
-- existente é afetada.
alter table tournaments drop constraint tournaments_starts_at_required;
