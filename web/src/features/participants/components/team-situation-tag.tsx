import { getTranslations } from "next-intl/server";

import { getCurrentUser } from "@/features/auth/server";
import { getFormation } from "@/features/teams/server";
import type { components } from "@/shared/api/schema";
import { StatusTag } from "@/shared/ui/status-tag";
import { SITUATION_STATUS } from "../lib/situation";

type Props = {
  tenant: string;
  slug: string;
  /** A situação pelo status e pelo check-in, que vale se quem vê está num time. */
  situation: components["schemas"]["Situation"];
};

/**
 * A etiqueta de "Sua situação" no TDM antes do início: com algum time já
 * formado e quem vê fora de todos, "Reserva, sem time" no lugar de
 * `situation` (quem se inscreve ou faz o check-in depois de formados os
 * times fica de reserva, e "Check-in feito" sozinho daria a entender que
 * está na chave). Lê a formação e a sessão: fica em `<Suspense>`, com a
 * etiqueta de `situation` enquanto carrega.
 */
export async function TeamSituationTag({ tenant, slug, situation }: Props) {
  const [t, formation, user] = await Promise.all([
    getTranslations("Participants.situation"),
    getFormation(tenant, slug),
    getCurrentUser(),
  ]);
  const reserve =
    formation !== undefined &&
    user !== undefined &&
    formation.teams.length > 0 &&
    !formation.teams.some((team) =>
      team.members.some((member) => member.user_id === user.id),
    );
  const shown = reserve ? "reserve" : situation;
  return (
    <StatusTag status={SITUATION_STATUS[shown]}>
      {t(shown, { place: "" })}
    </StatusTag>
  );
}
