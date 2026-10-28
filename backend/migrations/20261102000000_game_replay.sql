-- Mapa anulado pelo organizador para os jogadores o jogarem de novo (decisão
-- em "Resultados" no docs/decisions.md): o evento 'replay', sempre de um mapa
-- e sem vencedor. Os dados existentes não mudam: as regras só passam a
-- aceitar o tipo novo.
alter table match_events
    drop constraint match_events_kind_check,
    add constraint match_events_kind_check check (kind in (
        'reported', 'confirmed', 'auto_confirmed', 'contested', 'resolved',
        'entered', 'corrected', 'walkover', 'auto_walkover', 'voided', 'replay', 'decided',
        'reopened')),
    drop constraint match_events_outcome_check,
    add constraint match_events_outcome_check check (case
        when game is not null then
            kind in ('reported', 'confirmed', 'auto_confirmed', 'contested', 'resolved',
                     'entered', 'corrected', 'voided', 'replay')
            and winner_maps is null
            and (winner is not null) = (kind in ('reported', 'resolved', 'entered', 'corrected'))
        when kind = 'replay' then false
        when kind in ('reported', 'resolved', 'entered', 'corrected', 'decided')
            then winner is not null and winner_maps is not null
        when kind in ('walkover', 'auto_walkover') then winner is not null and winner_maps is null
        else winner is null and winner_maps is null
    end);
