import { getTranslations } from "next-intl/server";

import { BracketSkeleton } from "@/features/matches/components/bracket-skeleton";

/** A Chave carregando: a dica e as rodadas. */
export default async function Loading() {
  const t = await getTranslations("Bracket");
  return <BracketSkeleton label={t("loading")} />;
}
