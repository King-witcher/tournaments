import { useTranslations } from "next-intl";

import type { components } from "@/shared/api/schema";

/**
 * A modalidade como as listas e o cabeçalho a mostram: "Duelo" ou, no TDM,
 * com o tamanho ("TDM 2v2"). A lista de torneios (Server Component async)
 * monta o mesmo texto com `getTranslations`.
 */
export function useModeLabel() {
  const t = useTranslations("Tournaments");
  return (mode: components["schemas"]["TournamentMode"], teamSize: number) =>
    mode === "tdm" ? t("modeTeams", { size: teamSize }) : t(`mode.${mode}`);
}
