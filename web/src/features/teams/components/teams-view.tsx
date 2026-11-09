import { getTranslations } from "next-intl/server";

import { participantsEditable } from "@/features/tournaments/lib/status";
import { loadTournament } from "@/features/tournaments/server";
import { apiErrorMessage } from "@/shared/api/errors";
import { PageError } from "@/shared/ui/page-error";
import { checkInShown } from "../lib/formation";
import { getFormation } from "../server";
import { FormationEditor } from "./formation-editor";
import { TeamsSection } from "./teams-section";

type Props = { tenant: string; slug: string };

/**
 * A seção Times. Para a organização, antes de iniciar, a formação editável
 * (`FormationEditor`); depois de iniciar ou para quem joga, só leitura. No
 * duelo não há times: o aviso e o caminho de volta. Lê a sessão, então fica
 * em `<Suspense>`.
 */
export async function TeamsView({ tenant, slug }: Props) {
  const [tNav, loaded] = await Promise.all([
    getTranslations("Nav"),
    loadTournament(tenant, slug),
  ]);
  if (loaded.kind !== "ok") return null;

  const { me, tournament } = loaded;
  const formation =
    tournament.mode === "tdm" ? await getFormation(tenant, slug) : undefined;
  if (!formation) {
    // No duelo não há times; no TDM, a API falhou.
    const code = tournament.mode === "tdm" ? "internal" : "tournament.not_tdm";
    return (
      <PageError href={`/${tenant}/${slug}`} label={tNav("backToTournament")}>
        {await apiErrorMessage(code)}
      </PageError>
    );
  }

  const { status } = tournament;
  if (me.role !== "player" && participantsEditable(status)) {
    return (
      <FormationEditor
        tenant={tenant}
        tournament={slug}
        initial={formation}
        checkIn={checkInShown(status, formation)}
      />
    );
  }
  return <TeamsSection tenant={tenant} slug={slug} />;
}
