-- A aba que pediu a mudança ("Eco de quem agiu" em docs/decisions/realtime.md):
-- a página que aplica a resposta da ação manda o id da própria conexão do
-- WebSocket (`x-realtime-connection`), e as mudanças de partida que a ação
-- emitiu não voltam a essa conexão (ela recarregaria o que já tem). As outras
-- abas, inclusive as da mesma pessoa, recebem tudo.
--
-- Dados existentes: coluna nova e opcional; as linhas do outbox são de
-- passagem (o despachante apaga) e ficam sem origem.
alter table outbox
    -- O id que a aba deu à conexão (`/ws?connection=`); nulo fora de uma
    -- requisição (agendador, relógio do veto) ou sem o cabeçalho.
    add column origin text
        constraint outbox_origin_format check (origin ~ '^[A-Za-z0-9_-]{16,64}$');
