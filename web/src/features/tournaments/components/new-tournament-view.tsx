import { getTranslations } from "next-intl/server";

import { LoginPrompt } from "@/features/auth/components/login-prompt";
import { loadMaps } from "@/features/matches/server";
import { apiErrorMessage } from "@/shared/api/errors";
import { serverApi } from "@/shared/api/server";
import { PageError } from "@/shared/ui/page-error";
import { PageHeader } from "@/shared/ui/page-header";
import { NewTournamentForm } from "./new-tournament-form";

/**
 * Criar torneio na comunidade, para organizer e owner: o cabeçalho com a
 * trilha e o formulário. Lê a sessão, então fica em `<Suspense>`.
 */
export async function NewTournamentView({ tenant }: { tenant: string }) {
  const api = await serverApi();
  const [t, tNav, result] = await Promise.all([
    getTranslations("NewTournament"),
    getTranslations("Nav"),
    api
      .GET("/tenants/{slug}", { params: { path: { slug: tenant } } })
      .catch(() => undefined),
  ]);

  if (result?.response.status === 401) {
    return (
      <>
        <PageHeader title={t("title")} />
        <LoginPrompt message={t("loginRequired")} />
      </>
    );
  }
  const me = result?.data;
  if (!me) {
    return (
      <>
        <PageHeader title={t("title")} />
        <PageError href="/" label={tNav("backHome")}>
          {await apiErrorMessage(result?.error?.error ?? "internal")}
        </PageError>
      </>
    );
  }

  return (
    <>
      <PageHeader
        breadcrumbs={[
          { href: "/", label: tNav("home") },
          { href: `/${tenant}`, label: me.name },
        ]}
        title={t("title")}
      />
      {me.role === "player" ? (
        <PageError href={`/${tenant}`} label={tNav("backToCommunity")}>
          {await apiErrorMessage("tenant.forbidden")}
        </PageError>
      ) : (
        <NewTournamentForm tenant={tenant} maps={await loadMaps()} />
      )}
    </>
  );
}
