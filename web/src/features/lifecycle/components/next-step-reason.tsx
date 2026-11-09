import { getTranslations } from "next-intl/server";

import type { components } from "@/shared/api/schema";
import { Link } from "@/shared/ui/link";
import { getStartCopy } from "../server";
import styles from "./next-step-reason.module.scss";

type Tournament = components["schemas"]["Tournament"];

type Props = {
  tenant: string;
  tournament: Tournament;
};

/**
 * Por que o próximo passo não dá agora, escrito no estado do torneio (no
 * cabeçalho, o motivo é só a dica do botão): o que falta para iniciar. No TDM,
 * com o link para formar os times. Nada a dizer, nada aparece. Lê a sessão,
 * então fica em `<Suspense>`.
 */
export async function NextStepReason({ tenant, tournament }: Props) {
  const [tTeam, reason] = await Promise.all([
    getTranslations("Team"),
    nextStepReason(tenant, tournament),
  ]);
  if (!reason) return null;
  return (
    <>
      <p className={styles.reason}>{reason.text}</p>
      {reason.teams && (
        <Link href={`/${tenant}/${tournament.slug}/teams`}>
          {tTeam("form")}
        </Link>
      )}
    </>
  );
}

async function nextStepReason(tenant: string, tournament: Tournament) {
  const { status } = tournament;
  if (
    status === "check_in" ||
    (status === "registration" && !tournament.check_in)
  ) {
    const { disabledReason, note } = await getStartCopy(tenant, tournament);
    const text = disabledReason ?? note;
    return text ? { text, teams: tournament.mode === "tdm" } : undefined;
  }
  return undefined;
}
