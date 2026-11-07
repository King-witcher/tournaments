"use client";

import { useTranslations } from "next-intl";

import type { RulesImages } from "@/features/images/lib/rules-images";

import { useLanguagesSection } from "../hooks/use-languages-section";
import { useTournamentSettings } from "../hooks/use-tournament-settings";
import { isFinished } from "../lib/status";
import { SettingsSection } from "./settings-section";
import { TournamentTranslation } from "./tournament-translation";

/**
 * Outros idiomas: um bloco por idioma diferente do original, cada um sem
 * tradução ("Adicionar") ou com nome, descrição e regras ("Remover"), e um
 * Salvar para todos; no título, as traduções salvas. Encerrado o torneio, as
 * regras ficam só para leitura.
 */
export function SettingsLanguagesSection({
  rulesImages,
}: {
  /** As imagens das regras (o CDN e o tamanho máximo); `null` sem o bucket. */
  rulesImages: RulesImages | null;
}) {
  const tSettings = useTranslations("TournamentSettings");
  const tLocales = useTranslations("Locales");
  const { tournament } = useTournamentSettings();
  const form = useLanguagesSection();
  const finished = isFinished(tournament.status);
  const saved = form.others.filter(form.isSaved);

  return (
    <SettingsSection
      section="languages"
      form={form}
      summary={
        saved.length > 0
          ? saved.map((l) => tLocales(l)).join(", ")
          : tSettings("noTranslations")
      }
      lock={finished ? tSettings("locks.rules") : undefined}
      onSave={form.submit}
    >
      {form.others.map((l) => (
        <TournamentTranslation
          key={l}
          locale={l}
          original={form.original}
          draft={form.draftOf(l)}
          onChange={form.onChange}
          errors={form.errorsOf(l)}
          saved={form.isSaved(l)}
          rulesReadOnly={finished}
          rulesImages={rulesImages}
        />
      ))}
    </SettingsSection>
  );
}
