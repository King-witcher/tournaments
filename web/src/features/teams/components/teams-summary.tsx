import { getTranslations } from "next-intl/server";

import { hasStarted } from "@/features/tournaments/lib/status";
import type { components } from "@/shared/api/schema";
import { ButtonLink } from "@/shared/ui/button-link";
import { Section } from "@/shared/ui/section";
import { checkInShown, formationCounts } from "../lib/formation";
import { getFormation } from "../server";
import styles from "./teams-summary.module.scss";

type Props = {
  tenant: string;
  tournament: components["schemas"]["Tournament"];
};

/**
 * Na aba Organização do TDM, antes da ordem da chave: quantos times estão
 * completos e incompletos e quantos reservas (a regra de iniciar já está no
 * quadro do estado, ao lado do botão), com "Formar times" (a página
 * dos times). Depois de iniciar, onde os times estão e onde se substitui um
 * jogador. Fica em `<Suspense>`.
 */
export async function TeamsSummary({ tenant, tournament }: Props) {
  const t = await getTranslations("Team");
  const path = `/${tenant}/${tournament.slug}`;

  if (hasStarted(tournament.status)) {
    return (
      <Section
        title={t("manage.title")}
        action={
          <ButtonLink href={`${path}/participants`} kind="tertiary" size="sm">
            {t("manage.view")}
          </ButtonLink>
        }
      >
        <p className={styles.text}>{t("manage.started")}</p>
      </Section>
    );
  }

  const formation = await getFormation(tenant, tournament.slug);
  const checkIn = formation
    ? checkInShown(tournament.status, formation)
    : false;
  return (
    <Section
      title={t("manage.title")}
      action={
        <ButtonLink href={`${path}/teams`} kind="tertiary" size="sm">
          {t("form")}
        </ButtonLink>
      }
    >
      <p className={styles.text}>
        {formation
          ? t("summary.counts", formationCounts(formation, checkIn))
          : t("manage.unavailable")}
      </p>
    </Section>
  );
}
