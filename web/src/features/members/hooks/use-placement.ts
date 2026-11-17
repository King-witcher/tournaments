import { useTranslations } from "next-intl";

import type { components } from "@/shared/api/schema";

/**
 * A colocação no idioma de quem vê: "3º" ou, com empate, a faixa ("3º–4º").
 * Serve a Server e Client Components.
 */
export function usePlacement() {
  const t = useTranslations("Member.tournaments");
  return (placement: components["schemas"]["MyPlacement"]) =>
    placement.tied > 1
      ? t("range", {
          from: placement.rank,
          to: placement.rank + placement.tied - 1,
        })
      : t("rank", { rank: placement.rank });
}
