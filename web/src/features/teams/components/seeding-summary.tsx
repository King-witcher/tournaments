import { getTranslations } from "next-intl/server";

import { hasStarted, seedingOpen } from "@/features/tournaments/lib/status";
import type { components } from "@/shared/api/schema";
import { ButtonLink } from "@/shared/ui/button-link";
import { Section } from "@/shared/ui/section";
import { getSeeding } from "../server";
import styles from "./seeding-summary.module.scss";

type Props = {
  tenant: string;
  tournament: components["schemas"]["Tournament"];
};

/**
 * Na aba Organização, o resumo da ordem da chave (o seeding): o que ela
 * decide, quantos entram na chave e se a ordem já foi definida ou será
 * sorteada ao iniciar, com "Definir a ordem" (a página inteira) enquanto ela
 * se revisa. Antes disso e depois de iniciar, só diz quando ela vale. O nome
 * "seeding" sozinho não diz nada a quem não organiza torneios. Fica em
 * `<Suspense>`.
 */
export async function SeedingSummary({ tenant, tournament }: Props) {
  const t = await getTranslations("Participants.seeding");
  const hasCheckIn = tournament.check_in_minutes != null;

  if (!seedingOpen(tournament)) {
    return (
      <Section title={t("summary.title")} description={t("summary.what")}>
        <p className={styles.text}>
          {hasStarted(tournament.status)
            ? t("summary.started")
            : hasCheckIn
              ? t("summary.atCheckIn")
              : t("summary.atRegistration")}
        </p>
      </Section>
    );
  }

  const order = await getSeeding(tenant, tournament.slug);
  const count = order?.length ?? 0;
  // No TDM, a ordem é de times (os completos), não de inscritos.
  const teams = tournament.mode === "tdm";
  const text = !order
    ? t("summary.unavailable")
    : count === 0
      ? teams
        ? t("summary.emptyTeams")
        : hasCheckIn
          ? t("empty")
          : t("emptyNoCheckIn")
      : order.every((p) => p.seed == null)
        ? t("summary.random")
        : order.some((p) => p.seed == null)
          ? t("summary.partial")
          : t("summary.set");

  return (
    <Section
      title={t("summary.title")}
      description={t("summary.what")}
      action={
        count > 0 && (
          <ButtonLink
            href={`/${tenant}/${tournament.slug}/seeding`}
            kind="tertiary"
            size="sm"
          >
            {t("edit")}
          </ButtonLink>
        )
      }
    >
      <p className={styles.text}>
        {order &&
          `${
            teams
              ? t("summary.countTeams", { count })
              : t("summary.count", { count, total: tournament.participants })
          } `}
        {text}
      </p>
    </Section>
  );
}
