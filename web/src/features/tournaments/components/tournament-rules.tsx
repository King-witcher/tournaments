import { useTranslations } from "next-intl";

import { Markdown } from "@/shared/ui/markdown";

/** Regras do torneio, em Markdown, já no idioma de quem vê. */
export function TournamentRules({ rules }: { rules: string }) {
  const t = useTranslations("Tournament");
  return (
    <section className="flex flex-col gap-3">
      <h2 className="text-xl font-semibold">{t("rules")}</h2>
      <Markdown>{rules}</Markdown>
    </section>
  );
}
