import { getTranslations } from "next-intl/server";

import { getCurrentUser } from "@/features/auth/server";
import { getFormation } from "@/features/teams/server";
import { SituationMark } from "./situation-mark";

type Props = {
  tenant: string;
  slug: string;
  /** A situação pelo status e pelo check-in, que vale se quem vê está num time. */
  situation: "registered" | "checked_in";
};

/**
 * A situação de "Sua situação" no TDM antes do início: com algum time já
 * formado e quem vê fora de todos, a etiqueta "Reserva, sem time" no lugar
 * do texto de `situation` (quem se inscreve ou faz o check-in depois de
 * formados os times fica de reserva, e "Check-in feito" sozinho daria a
 * entender que está na chave). Lê a formação e a sessão: fica em
 * `<Suspense>`, com o texto de `situation` enquanto carrega.
 */
export async function TeamSituation({ tenant, slug, situation }: Props) {
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
  return <SituationMark situation={shown} label={t(shown, { place: "" })} />;
}
