import { getLocale } from "next-intl/server";

import { loadTournament } from "@/features/tournaments/server";
import { redirect } from "@/i18n/navigation";
import { preload } from "@/shared/lib/preload";
import { getParticipants } from "../server";
import { ParticipantTable } from "./participant-table";

type Props = { tenant: string; slug: string };

/**
 * A aba Inscritos: a tabela das inscrições individuais, em todo modo (os
 * times têm a seção deles). Fica em `<Suspense>`.
 */
export async function ParticipantsView({ tenant, slug }: Props) {
  preload(getParticipants(tenant, slug));
  const loaded = await loadTournament(tenant, slug);
  if (loaded.kind !== "ok") return null;
  // O rascunho não tem inscritos: a seção nem aparece no menu.
  if (loaded.tournament.status === "draft") {
    redirect({ href: `/${tenant}/${slug}`, locale: await getLocale() });
  }

  return <ParticipantTable tenant={tenant} slug={slug} />;
}
