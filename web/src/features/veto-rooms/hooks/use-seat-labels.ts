import { useTranslations } from "next-intl";

import type { SeatLabels } from "../lib/seat-names";

/** Os nomes traduzidos de quem ainda não entrou. */
export function useSeatLabels(): SeatLabels {
  const t = useTranslations("VetoRoom.seats");
  return {
    side: (side) => t("side", { side: side.toUpperCase() }),
    team: (name) => t("team", { name }),
    seat: (seat) =>
      t("seat", { side: seat.side.toUpperCase(), position: seat.position }),
    empty: t("empty"),
  };
}
