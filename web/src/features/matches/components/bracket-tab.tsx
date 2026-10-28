import { getTranslations } from "next-intl/server";

import { LoginPrompt } from "@/features/auth/components/login-prompt";
import { ErrorBack } from "@/features/tournaments/components/error-back";
import { loadTournament } from "@/features/tournaments/server";
import { ButtonLink } from "@/shared/ui/button-link";
import { EmptyState } from "@/shared/ui/empty-state";
import { BracketSection } from "./bracket-section";

type Props = { tenant: string; slug: string };

/**
 * A aba Chave do torneio: a chave e a classificação a partir do início;
 * antes, o aviso de quando ela aparece, com o caminho para os inscritos.
 * Lê a sessão: fica em `<Suspense>`.
 */
export async function BracketTab({ tenant, slug }: Props) {
  const [t, loaded] = await Promise.all([
    getTranslations("Bracket"),
    loadTournament(tenant, slug),
  ]);
  if (loaded.kind === "login") {
    return <LoginPrompt message={t("loginRequired")} />;
  }
  if (loaded.kind === "error") {
    return <ErrorBack tenant={tenant} code={loaded.code} />;
  }
  const { tournament } = loaded;
  if (tournament.status !== "in_progress" && tournament.status !== "finished") {
    return (
      <EmptyState
        action={
          <ButtonLink href={`/${tenant}/${slug}/participants`} kind="tertiary">
            {t("seeParticipants")}
          </ButtonLink>
        }
      >
        {t("notStarted")}
      </EmptyState>
    );
  }
  return <BracketSection tenant={tenant} tournament={tournament} />;
}
