import { Stack } from "@carbon/react";
import { getLocale, getTranslations } from "next-intl/server";

import { LoginPrompt } from "@/features/auth/components/login-prompt";
import { ErrorBack } from "@/features/tournaments/components/error-back";
import { localizedName } from "@/features/tournaments/lib/localize";
import { seedingOpen } from "@/features/tournaments/lib/status";
import { loadTournament } from "@/features/tournaments/server";
import { apiErrorMessage } from "@/shared/api/errors";
import { ButtonLink } from "@/shared/ui/button-link";
import { EmptyState } from "@/shared/ui/empty-state";
import { PageError } from "@/shared/ui/page-error";
import { PageHeader } from "@/shared/ui/page-header";
import { getSeeding } from "../server";
import { SeedingEditor } from "./seeding-editor";
import styles from "./seeding-view.module.scss";

type Props = { tenant: string; slug: string };

/**
 * A página do seeding, para organizer e owner, na revisão antes de iniciar:
 * quantos entram na chave, a explicação e a lista com a posição de cada um,
 * "Sortear" e "Salvar". Fora da revisão (ou para quem joga), o aviso e o
 * caminho de volta. Lê a sessão, então fica em `<Suspense>`.
 */
export async function SeedingView({ tenant, slug }: Props) {
  const [t, tNav, tTournament, locale, loaded] = await Promise.all([
    getTranslations("Participants.seeding"),
    getTranslations("Nav"),
    getTranslations("Tournament"),
    getLocale(),
    loadTournament(tenant, slug),
  ]);
  if (loaded.kind === "login") {
    return <LoginPrompt message={tTournament("loginRequired")} />;
  }
  if (loaded.kind === "error") {
    return <ErrorBack tenant={tenant} code={loaded.code} />;
  }

  const { me, tournament } = loaded;
  const path = `/${tenant}/${slug}`;
  const manage = `${path}/manage`;
  const breadcrumbs = [
    { href: "/", label: tNav("home") },
    { href: `/${tenant}`, label: me.name },
    { href: path, label: localizedName(tournament, locale) },
  ];

  if (me.role === "player") {
    return (
      <>
        <PageHeader breadcrumbs={breadcrumbs} title={t("pageTitle")} />
        <PageError href={path} label={tNav("backToTournament")}>
          {await apiErrorMessage("tenant.forbidden")}
        </PageError>
      </>
    );
  }
  const order = seedingOpen(tournament)
    ? await getSeeding(tenant, slug)
    : undefined;
  if (!order) {
    // Fora da revisão (rascunho, inscrição com check-in, depois de iniciar)
    // ou a API falhou.
    return (
      <>
        <PageHeader breadcrumbs={breadcrumbs} title={t("pageTitle")} />
        <PageError href={manage} label={t("backToManage")}>
          {seedingOpen(tournament)
            ? await apiErrorMessage("internal")
            : t("closed")}
        </PageError>
      </>
    );
  }

  const hasCheckIn = tournament.check_in_minutes != null;
  return (
    <>
      <PageHeader
        breadcrumbs={breadcrumbs}
        title={t("title", {
          count: order.length,
          total: tournament.participants,
        })}
      >
        <Stack gap={3} className={styles.hint}>
          <p>{t("what")}</p>
          <p>{t("hint")}</p>
        </Stack>
      </PageHeader>
      {order.length === 0 ? (
        <EmptyState
          action={
            <ButtonLink href={manage} kind="tertiary">
              {t("backToManage")}
            </ButtonLink>
          }
        >
          {hasCheckIn ? t("empty") : t("emptyNoCheckIn")}
        </EmptyState>
      ) : (
        <SeedingEditor
          // Recomeça do que a API devolveu a cada atualização da página.
          key={order.map((p) => `${p.user_id}:${p.seed}`).join(",")}
          tenant={tenant}
          tournament={slug}
          initial={order}
        />
      )}
    </>
  );
}
