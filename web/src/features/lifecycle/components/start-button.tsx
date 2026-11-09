import { getTranslations } from "next-intl/server";

import type { components } from "@/shared/api/schema";
import { getStartCopy } from "../server";
import { TransitionButton } from "./transition-button";

type Props = {
  tenant: string;
  tournament: components["schemas"]["Tournament"];
};

/**
 * "Iniciar torneio" com a conta na confirmação e, quando não dá, o motivo
 * (`getStartCopy`).
 */
export async function StartButton({ tenant, tournament }: Props) {
  const [t, copy] = await Promise.all([
    getTranslations("Lifecycle"),
    getStartCopy(tenant, tournament),
  ]);
  return (
    <TransitionButton
      tenant={tenant}
      slug={tournament.slug}
      transition="start"
      label={t("start")}
      {...copy}
    />
  );
}
