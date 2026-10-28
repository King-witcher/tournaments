import { getTranslations } from "next-intl/server";

import { BracketSkeleton } from "@/features/matches/components/bracket-skeleton";

/** A aba Chave carregando: as rodadas e a classificação. */
export default async function Loading() {
  const t = await getTranslations("Bracket");
  return <BracketSkeleton label={t("loading")} />;
}
