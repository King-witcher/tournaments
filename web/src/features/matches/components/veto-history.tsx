import { AccordionItem } from "@carbon/react";
import { useTranslations } from "next-intl";

import { shownNickname } from "@/features/members/lib/nickname";
import type { components } from "@/shared/api/schema";
import { LocalDateTime } from "@/shared/ui/local-date-time";
import rowStyles from "./match-event-row.module.scss";
import styles from "./match-history.module.scss";

type Schemas = components["schemas"];

type Props = {
  veto: Schemas["VetoView"];
  names: { a: string; b: string };
};

/**
 * O bloco fechado "Histórico do veto": cada passo, com quem, quando e se foi
 * sorteado pelo fim do tempo, inclusive os desfeitos. Vai dentro de um
 * `Accordion`.
 */
export function VetoHistory({ veto, names }: Props) {
  const t = useTranslations("Match.veto.history");
  const tGame = useTranslations("Match.games");
  const tMember = useTranslations("Member");
  if (veto.history.length === 0) return null;
  const teams = veto.team_size > 1;

  return (
    <AccordionItem title={t("title")}>
      <ol className={styles.events}>
        {veto.history.map((e, i) => {
          // No pick por jogador, o jogador (também no sorteado); no passo do
          // lado, nos times, o time e quem agiu por ele.
          const actor =
            e.actor != null
              ? shownNickname(e.actor, tMember("deletedAccount"))
              : null;
          const who =
            e.player != null && e.player !== ""
              ? shownNickname(e.player, tMember("deletedAccount"))
              : e.side
                ? teams && actor
                  ? `${names[e.side]} (${actor})`
                  : names[e.side]
                : "";
          const text = t(`kinds.${e.kind}`, {
            name: who,
            actor: actor ?? "",
            map: e.map?.name ?? "",
            champion: e.champion?.name ?? "",
            game: e.game ?? 1,
            forced: e.actor != null ? "yes" : "no",
          });
          // O pick de campeão diz a partida a que se destina; o ban de
          // campeão e o veto de mapas valem para a série inteira (o mapa
          // escolhido para a partida 2 sai antes da partida 1: "Partida 1"
          // na frente enganaria).
          const game = e.pick_game ?? e.game;
          const seriesWide =
            e.kind === "game" ||
            e.kind === "begin" ||
            (e.kind === "ban" && e.champion != null) ||
            (e.map != null && veto.map_mode === "veto");
          // O capitão escolheu por um colega: o campeão é do colega.
          const pickedBy =
            e.player && actor && e.player !== e.actor
              ? t("pickedBy", { actor })
              : null;
          const parts = [
            game != null && !seriesWide
              ? tGame("game", { number: game })
              : null,
            text,
            pickedBy,
            e.timeout ? t("timeout") : null,
          ].filter(Boolean);
          return (
            // Os eventos não mudam de ordem: o índice é estável.
            // biome-ignore lint/suspicious/noArrayIndexKey: lista só cresce
            <li key={i} className={rowStyles.event}>
              <span className={rowStyles.time}>
                <LocalDateTime value={e.at} />
              </span>
              <span>{parts.join(" · ")}</span>
            </li>
          );
        })}
      </ol>
    </AccordionItem>
  );
}
