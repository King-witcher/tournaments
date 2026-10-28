import { getTranslations } from "next-intl/server";

import { LoginPrompt } from "@/features/auth/components/login-prompt";
import { Link } from "@/i18n/navigation";
import { apiErrorMessage } from "@/shared/api/errors";
import { Alert } from "@/shared/ui/alert";
import { textsEditable } from "../lib/status";
import { loadTournament } from "../server";
import { TournamentForm } from "./tournament-form";
import { TranslationSection } from "./translation-section";

type Props = { tenant: string; slug: string };

/** Editar torneio e traduções, para organizer e owner. Fica em `<Suspense>`. */
export async function EditTournamentView({ tenant, slug }: Props) {
  const [t, loaded] = await Promise.all([
    getTranslations("Tournament"),
    loadTournament(tenant, slug),
  ]);
  if (loaded.kind === "login") {
    return <LoginPrompt message={t("loginRequired")} />;
  }
  if (loaded.kind === "error") {
    return <Alert>{await apiErrorMessage(loaded.code)}</Alert>;
  }
  const { me, tournament } = loaded;
  if (me.role === "player") {
    return <Alert>{await apiErrorMessage("tenant.forbidden")}</Alert>;
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
        <Alert>{await apiErrorMessage("tournament.finished")}</Alert>
      )}
    </>
  );
}
