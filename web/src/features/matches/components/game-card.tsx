import { useTranslations } from "next-intl";

import { GameScreenshots } from "@/features/screenshots/components/game-screenshots";
import type { components } from "@/shared/api/schema";
import { Accordion } from "@/shared/ui/accordion";
import { Layer } from "@/shared/ui/carbon/layer";
import { Tile } from "@/shared/ui/carbon/tile";
import { gameRowId, type MatchTarget } from "../lib/match-target";
import styles from "./game-card.module.scss";
import { GameCardGate } from "./game-card-gate";
import { GameForm } from "./game-form";
import { GameVersus } from "./game-versus";

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
 * A partida da vez num cartão: o alto (os lados, o mapa e os campeões) e,
 * embaixo, o envio. Para quem joga e pode enviar, o formulário depois do veto
 * da partida (no mapa que ele deu, com os frags embaixo de cada lado e a captura
 * de tela no fim; sem veto, com a escolha do mapa); para os outros, os
 * screenshots e "aguardando envio" (ao colega, que é o capitão quem envia) e,
 * para a organização que não envia, o lançamento fechado num bloco (é exceção).
 * Durante a votação, sem nada a
 * mostrar ainda, quem joga não vê o cartão (`GameCardGate`, pela série ao
 * vivo).
 */
export function GameCard({ target, detail, number, names, maps }: Props) {
  const t = useTranslations("Match.games");
  const tForm = useTranslations("Match.form");
  const plays = detail.viewer_side != null;
  const sends = detail.can_report;
  const enters = detail.can_manage && !sends;
  const veto = detail.veto;
  // Sem o veto da partida (a votação, no passo), nem se sabe o mapa.
  const reportable = !veto || veto.reportable;
  const fixedMap = veto?.games.find((g) => g.game === number)?.map;
  const screenshots = (
    <GameScreenshots
      target={target}
      number={number}
      reporting={sends}
      focusId={gameRowId(number)}
    />
  );

  return (
    <GameCardGate number={number}>
      <Tile>
        <Layer className={styles.content}>
          <GameVersus number={number} />
          {sends && reportable && (
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
              card={fixedMap != null}
            />
          )}
          {!sends && reportable && screenshots}
          {!sends && (
            <p className={styles.awaiting}>
              {t(plays ? "captainReports" : "awaiting")}
            </p>
          )}
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
    </GameCardGate>
  );
}
