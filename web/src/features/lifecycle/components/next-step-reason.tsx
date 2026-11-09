import { getTranslations } from "next-intl/server";

import { startTiming } from "@/features/tournaments/lib/form-values";
import type { components } from "@/shared/api/schema";
import { Link } from "@/shared/ui/link";
import { getStartCopy } from "../server";
import styles from "./next-step-reason.module.scss";

type Tournament = components["schemas"]["Tournament"];

type Props = {
  tenant: string;
  tournament: Tournament;
  /** Referência para saber se a data de início ficou para trás. */
  now: Date;
};

/**
 * Por que o próximo passo não dá agora, escrito no estado do torneio (no
 * cabeçalho, o motivo é só a dica do botão): a data do rascunho que ficou para
 * trás ou o que falta para iniciar. No TDM, com o link para formar os times.
 * Nada a dizer, nada aparece. Lê a sessão, então fica em `<Suspense>`.
 */
export async function NextStepReason({ tenant, tournament, now }: Props) {
  const [tTeam, reason] = await Promise.all([
    getTranslations("Team"),
    nextStepReason(tenant, tournament, now),
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

async function nextStepReason(
  tenant: string,
  tournament: Tournament,
  now: Date,
) {
  const { status } = tournament;
  const hasCheckIn = tournament.check_in_minutes != null;
  if (status === "draft") {
    const behind = startTiming(
      tournament.starts_at ?? null,
      tournament.check_in_minutes ?? null,
      now,
    );
    if (!behind) return undefined;
    const t = await getTranslations("Tournament");
    return { text: t(`publishBlocked.${behind}`), teams: false };
  }
  if (status === "check_in" || (status === "registration" && !hasCheckIn)) {
    const { disabledReason, note } = await getStartCopy(tenant, tournament);
    const text = disabledReason ?? note;
    return text ? { text, teams: tournament.mode === "tdm" } : undefined;
  }
  return undefined;
}
