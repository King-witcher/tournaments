"use client";

import { useTranslations } from "next-intl";

import { useLanguagesSection } from "../hooks/use-languages-section";
import { useTournamentSettings } from "../hooks/use-tournament-settings";
import { isFinished } from "../lib/status";
import { SettingsSection } from "./settings-section";
import { TournamentTranslation } from "./tournament-translation";

/**
 * Outros idiomas: um bloco por idioma diferente do original, cada um sem
 * tradução ("Adicionar") ou com nome, descrição e regras ("Remover"), e um
 * Salvar para todos. Encerrado o torneio, as regras ficam só para leitura.
 */
export function SettingsLanguagesSection() {
  const tSettings = useTranslations("TournamentSettings");
  const { tournament } = useTournamentSettings();
  const form = useLanguagesSection();
  const finished = isFinished(tournament.status);

  return (
    <SettingsSection
      section="languages"
      form={form}
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
        />
      ))}
    </SettingsSection>
  );
}
