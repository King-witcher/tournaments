import { getTranslations } from "next-intl/server";

import { LoginPrompt } from "@/features/auth/components/login-prompt";
import { loadMaps } from "@/features/matches/server";
import { apiErrorMessage } from "@/shared/api/errors";
import { serverApi } from "@/shared/api/server";
import { PageError } from "@/shared/ui/page-error";
import { PageHeader } from "@/shared/ui/page-header";
import { NewTournamentForm } from "./new-tournament-form";

type Props = {
  tenant: string;
  /** O slug do torneio de onde copiar (`?from=`). */
  from: string | undefined;
};

/**
 * Criar torneio na comunidade, para organizer e owner: o cabeçalho com a
 * trilha e o formulário, que começa da cópia de `from` quando ele existe na
 * comunidade (senão, do padrão). Lê a sessão, então fica em `<Suspense>`.
 */
export async function NewTournamentView({ tenant, from }: Props) {
  const api = await serverApi();
  const [t, tNav, result, tournaments, source] = await Promise.all([
    getTranslations("NewTournament"),
    getTranslations("Nav"),
    api
      .GET("/tenants/{slug}", { params: { path: { slug: tenant } } })
      .catch(() => undefined),
    api
      .GET("/tenants/{slug}/tournaments", {
        params: { path: { slug: tenant } },
      })
      .then(({ data }) => data ?? [])
      .catch(() => []),
    from
      ? api
          .GET("/tenants/{slug}/tournaments/{tournament}", {
            params: { path: { slug: tenant, tournament: from } },
          })
          .then(({ data }) => data)
          .catch(() => undefined)
      : undefined,
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
        <NewTournamentForm
          tenant={tenant}
          maps={await loadMaps()}
          tournaments={tournaments}
          source={source}
        />
      )}
    </>
  );
}
