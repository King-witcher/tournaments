import { getTranslations } from "next-intl/server";

import { TournamentFormSkeleton } from "@/features/tournaments/components/tournament-form-skeleton";
import { Page } from "@/shared/ui/page";

/** Editar torneio carregando, na forma da página: a trilha, o título e o formulário. */
export default async function Loading() {
  const t = await getTranslations("TournamentForm.edit");
  return (
    <Page>
      <TournamentFormSkeleton label={t("loading")} edit />
    </Page>
  );
}
