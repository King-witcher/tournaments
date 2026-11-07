import { getTranslations } from "next-intl/server";

import { LoginPrompt } from "@/features/auth/components/login-prompt";
import { StageSelect } from "@/features/stages/components/stage-select";
import {
  bracketStage,
  currentStage,
  hasSeveralStages,
  type StageParam,
  standingsStage,
} from "@/features/stages/lib/stages";
import { ErrorBack } from "@/features/tournaments/components/error-back";
import { loadTournament } from "@/features/tournaments/server";
import { ButtonLink } from "@/shared/ui/button-link";
import { Stack } from "@/shared/ui/carbon/stack";
import { EmptyState } from "@/shared/ui/empty-state";
import { BracketSection } from "./bracket-section";
import { StandingsSection } from "./standings-section";

type Props = {
  tenant: string;
  slug: string;
  /** Qual das abas que saem da chave. */
  view: "bracket" | "standings";
  /** A fase pedida no endereço (`?stage=`); sem ela, a atual. */
  stage?: StageParam;
};

/**
 * As abas Chave e Classificação do torneio, a partir do início; antes, o
 * aviso de quando elas têm conteúdo, com o caminho para os inscritos. Com
 * mais de uma fase começada, um seletor de fase em cima (a atual é a
 * padrão). Lê a sessão: fica em `<Suspense>`.
 */
export async function BracketTab({ tenant, slug, view, stage }: Props) {
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
        {t(view === "standings" ? "standings.notStarted" : "notStarted")}
      </EmptyState>
    );
  }

  // A fase mostrada; a atual vai sem número à API, para dividir a busca com o
  // resto da página.
  const shown =
    view === "standings"
      ? standingsStage(tournament, stage)
      : bracketStage(tournament, stage);
  const requested =
    shown === "final" || shown === currentStage(tournament).number
      ? undefined
      : shown;
  const content =
    view === "standings" ? (
      <StandingsSection
        tenant={tenant}
        tournament={tournament}
        organizer={loaded.me.role !== "player"}
        stage={requested}
        final={shown === "final"}
      />
    ) : (
      <BracketSection
        tenant={tenant}
        tournament={tournament}
        stage={requested}
      />
    );
  if (!hasSeveralStages(tournament)) return content;
  return (
    <Stack gap={7}>
      <StageSelect tournament={tournament} view={view} value={shown} />
      {content}
    </Stack>
  );
}
