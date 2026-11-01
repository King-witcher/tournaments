import { Layer, Tile } from "@carbon/react";
import { useTranslations } from "next-intl";

import { GameScreenshots } from "@/features/screenshots/components/game-screenshots";
import type { components } from "@/shared/api/schema";
import { Accordion } from "@/shared/ui/accordion";
import { gameRowId, type MatchTarget } from "../lib/match-target";
import styles from "./game-card.module.scss";
import { GameCardHead } from "./game-card-head";
import { GameForm } from "./game-form";

type Schemas = components["schemas"];

type Props = {
  target: MatchTarget;
  detail: Schemas["MatchDetail"];
  /** A partida da vez (a próxima que dá para enviar). */
  number: number;
  names: { a: string; b: string };
  /** Os mapas que cabem nesta partida (sem os já jogados na série). */
  maps: Schemas["Map"][];
};

/**
 * A partida da vez num cartão: o alto (mapa, lados e campeões, votação) com
 * o veto mais novo e, embaixo, o envio. Para quem joga, o formulário aberto
 * depois do veto da partida (no mapa que ele deu, com os frags nas colunas
 * dos lados e a captura de tela no fim); para os outros, os screenshots e
 * "aguardando envio" e, para a organização que não joga, o lançamento
 * fechado num bloco (é exceção).
 */
export function GameCard({ target, detail, number, names, maps }: Props) {
  const t = useTranslations("Match.games");
  const tForm = useTranslations("Match.form");
  const plays = detail.viewer_side != null;
  const enters = detail.can_manage && !plays;
  const veto = detail.veto;
  // Sem o veto da partida (a votação, no alto), nem se sabe o mapa.
  const reportable = !veto || veto.reportable;
  const fixedMap = veto?.games.find((g) => g.game === number)?.map;
  const screenshots = (
    <GameScreenshots
      target={target}
      number={number}
      reporting={plays}
      focusId={gameRowId(number)}
    />
  );

  return (
    // tabIndex -1: recebe o foco depois de anular a partida com este número.
    <li id={gameRowId(number)} tabIndex={-1} className={styles.item}>
      <Tile className={styles.card}>
        <Layer className={styles.content}>
          <GameCardHead target={target} number={number} />
          {plays && reportable && (
            // A chave monta o formulário de novo a cada partida: o mapa e o
            // placar da anterior não vão para a seguinte.
            <GameForm
              key={number}
              target={target}
              number={number}
              mode="report"
              names={names}
              maps={maps}
              fixedMap={fixedMap ?? undefined}
              beforeSubmit={screenshots}
              submitLabel={tForm("submit", { number })}
              card
            />
          )}
          {!plays && reportable && screenshots}
          {!plays && <p className={styles.awaiting}>{t("awaiting")}</p>}
          {/* A chave reinicia o bloco (fechado) a cada partida nova. */}
          {enters && (
            <Accordion key={number} title={tForm("enter", { number })}>
              <GameForm
                target={target}
                number={number}
                mode="organizer"
                names={names}
                maps={maps}
                submitLabel={tForm("enter", { number })}
              />
            </Accordion>
          )}
        </Layer>
      </Tile>
    </li>
  );
}
