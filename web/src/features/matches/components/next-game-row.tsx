import { useTranslations } from "next-intl";

import { GameScreenshots } from "@/features/screenshots/components/game-screenshots";
import type { components } from "@/shared/api/schema";
import { Accordion } from "@/shared/ui/accordion";
import { gameRowId, type MatchTarget } from "../lib/match-target";
import { GameForm } from "./game-form";
import styles from "./next-game-row.module.scss";

type Schemas = components["schemas"];

type Props = {
  target: MatchTarget;
  number: number;
  names: { a: string; b: string };
  maps: Schemas["Map"][];
  /** Quem vê joga a partida: envia como jogador. */
  plays: boolean;
  /** Organizer ou owner que não joga: lança, já confirmado. */
  enters: boolean;
  /**
   * Com veto no torneio: se o veto deste mapa acabou (o jogador só envia
   * depois) e o mapa que ele deu (o envio fica preso a ele).
   */
  veto?: { reportable: boolean; map: Schemas["MapRef"] | null } | null;
};

/**
 * A última linha da lista: o próximo mapa. Para quem joga, o formulário de
 * envio aberto (com veto, só depois do veto do mapa, e no mapa que ele deu),
 * com os screenshots do mapa antes dele (o anexo, quando o screenshot é
 * obrigatório); para os outros, os screenshots e "aguardando envio" e, para a
 * organização, o lançamento fechado num bloco (é exceção).
 */
export function NextGameRow({
  target,
  number,
  names,
  maps,
  plays,
  enters,
  veto,
}: Props) {
  const t = useTranslations("Match.games");
  const tForm = useTranslations("Match.form");

  return (
    // tabIndex -1: recebe o foco depois de anular o mapa com este número.
    <li id={gameRowId(number)} tabIndex={-1} className={styles.row}>
      <span className={styles.num}>
        {t.rich("numberCell", {
          number,
          prefix: (chunks) => (
            <span className={styles.numPrefix}>{chunks}</span>
          ),
        })}
      </span>
      <div className={styles.body}>
        {plays && veto && !veto.reportable ? (
          // O veto vem antes: sem ele, nem se sabe o mapa do screenshot.
          <p className={styles.awaiting}>{t("vetoFirst", { number })}</p>
        ) : (
          <>
            {/* Antes do envio: com screenshot obrigatório, ele vem primeiro. */}
            <GameScreenshots
              target={target}
              number={number}
              reporting={plays}
              focusId={gameRowId(number)}
            />
            {plays ? (
              <GameForm
                target={target}
                number={number}
                mode="report"
                names={names}
                maps={maps}
                fixedMap={veto?.map ?? undefined}
                submitLabel={tForm("submit", { number })}
              />
            ) : (
              <p className={styles.awaiting}>{t("awaiting")}</p>
            )}
          </>
        )}
        {/* A chave reinicia o bloco (fechado) a cada mapa novo. */}
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
      </div>
    </li>
  );
}
