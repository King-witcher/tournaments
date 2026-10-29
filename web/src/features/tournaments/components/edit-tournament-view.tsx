import { getLocale, getTranslations } from "next-intl/server";

import { LoginPrompt } from "@/features/auth/components/login-prompt";
import { loadMaps } from "@/features/matches/server";
import { apiErrorMessage } from "@/shared/api/errors";
import { PageError } from "@/shared/ui/page-error";
import { PageHeader } from "@/shared/ui/page-header";
import { localizedName } from "../lib/localize";
import { textsEditable } from "../lib/status";
import { loadTournament } from "../server";
import { EditTournamentForm } from "./edit-tournament-form";

type Props = { tenant: string; slug: string };

/**
 * Editar torneio e traduções, para organizer e owner: o cabeçalho com a
 * trilha até o torneio e o formulário. Sem permissão ou com o torneio
 * encerrado, o erro e a volta para o torneio. Fica em `<Suspense>`.
 */
export async function EditTournamentView({ tenant, slug }: Props) {
  const [t, tNav, locale, loaded] = await Promise.all([
    getTranslations("TournamentForm.edit"),
    getTranslations("Nav"),
    getLocale(),
    loadTournament(tenant, slug),
  ]);
  if (loaded.kind === "login") {
    return (
      <>
        <PageHeader title={t("title")} />
        <LoginPrompt message={t("loginRequired")} />
      </>
    );
  }
  if (loaded.kind === "error") {
    // A volta é para a comunidade, se ela ainda abre; senão, para o início.
    const outside =
      loaded.code.startsWith("tenant.") || loaded.code.startsWith("auth.");
    return (
      <>
        <PageHeader title={t("title")} />
        <PageError
          href={outside ? "/" : `/${tenant}`}
          label={outside ? tNav("backHome") : tNav("backToCommunity")}
        >
          {await apiErrorMessage(loaded.code)}
        </PageError>
      </>
    );
  }

  const { me, tournament } = loaded;
  const path = `/${tenant}/${slug}`;
  const blocked =
    me.role === "player"
      ? "tenant.forbidden"
      : textsEditable(tournament.status)
        ? undefined
        : "tournament.finished";

  return (
    <>
      <PageHeader
        breadcrumbs={[
          { href: "/", label: tNav("home") },
          { href: `/${tenant}`, label: me.name },
          { href: path, label: localizedName(tournament, locale) },
        ]}
        title={t("title")}
      />
      {blocked ? (
        <PageError href={path} label={tNav("backToTournament")}>
          {await apiErrorMessage(blocked)}
        </PageError>
      ) : (
        <EditTournamentForm
          tenant={tenant}
          tournament={tournament}
          maps={await loadMaps()}
        />
      )}
    </>
  );
}
