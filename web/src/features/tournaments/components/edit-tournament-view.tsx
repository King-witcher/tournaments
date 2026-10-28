import { getTranslations } from "next-intl/server";

import { LoginPrompt } from "@/features/auth/components/login-prompt";
import { Link } from "@/i18n/navigation";
import { apiErrorMessage } from "@/shared/api/errors";
import { PageError } from "@/shared/ui/page-error";
import { textsEditable } from "../lib/status";
import { loadTournament } from "../server";
import { ErrorBack } from "./error-back";
import { TournamentForm } from "./tournament-form";
import { TranslationSection } from "./translation-section";

type Props = { tenant: string; slug: string };

/** Editar torneio e traduções, para organizer e owner. Fica em `<Suspense>`. */
export async function EditTournamentView({ tenant, slug }: Props) {
  const [t, tNav, loaded] = await Promise.all([
    getTranslations("Tournament"),
    getTranslations("Nav"),
    loadTournament(tenant, slug),
  ]);
  if (loaded.kind === "login") {
    return <LoginPrompt message={t("loginRequired")} />;
  }
  if (loaded.kind === "error") {
    return <ErrorBack tenant={tenant} code={loaded.code} />;
  }
  const { me, tournament } = loaded;
  // Sem permissão ou encerrado: o erro e a volta para o torneio.
  const back = (message: string) => (
    <PageError href={`/${tenant}/${slug}`} label={tNav("backToTournament")}>
      {message}
    </PageError>
  );
  if (me.role === "player") {
    return back(await apiErrorMessage("tenant.forbidden"));
  }

  return (
    <>
      <header className="flex flex-col gap-1">
        <Link
          href={`/${tenant}/${slug}`}
          className="text-sm opacity-70 hover:underline"
        >
          {tournament.name}
        </Link>
        <h1 className="text-3xl font-semibold">{t("editTitle")}</h1>
      </header>
      {textsEditable(tournament.status) ? (
        <>
          <TournamentForm tenant={tenant} tournament={tournament} />
          <TranslationSection tenant={tenant} tournament={tournament} />
        </>
      ) : (
        back(await apiErrorMessage("tournament.finished"))
      )}
    </>
  );
}
