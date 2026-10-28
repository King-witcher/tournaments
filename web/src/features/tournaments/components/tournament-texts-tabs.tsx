import { Stack, Tab, TabList, TabPanel, TabPanels, Tabs } from "@carbon/react";
import { useTranslations } from "next-intl";

import type { components } from "@/shared/api/schema";
import type { TournamentFormState } from "../hooks/use-tournament-form";
import { TournamentTextFields } from "./tournament-text-fields";
import { TranslationPanel } from "./translation-panel";

type Locale = components["schemas"]["Locale"];

type Props = {
  form: TournamentFormState;
  /** A aba aberta (o formulário a troca quando um campo de outra é recusado). */
  language: Locale;
  onLanguageChange: (language: Locale) => void;
  /** Os idiomas que o torneio já tem traduzidos (na edição). */
  saved?: Locale[];
};

/**
 * Descrição e regras, uma aba por idioma: primeiro o original (na criação, o
 * da interface), depois os outros, que começam sem tradução. Embaixo de cada
 * idioma, se é o original, uma tradução ou se ainda não tem.
 */
export function TournamentTextsTabs({
  form,
  language,
  onLanguageChange,
  saved = [],
}: Props) {
  const t = useTranslations("TournamentForm");
  const tLocales = useTranslations("Locales");
  const languages = [form.original, ...form.others];
  const state = (l: Locale) =>
    l === form.original
      ? t("original")
      : form.translationOf(l).enabled
        ? t("translated")
        : t("untranslated");

  return (
    <Tabs
      selectedIndex={languages.indexOf(language)}
      onChange={({ selectedIndex }) =>
        onLanguageChange(languages[selectedIndex])
      }
    >
      <TabList aria-label={t("languages")} contained>
        {languages.map((l) => (
          <Tab key={l} secondaryLabel={state(l)}>
            {tLocales(l)}
          </Tab>
        ))}
      </TabList>
      <TabPanels>
        {languages.map((l) => (
          <TabPanel key={l}>
            <div data-language={l}>
              {l === form.original ? (
                <Stack gap={6}>
                  <TournamentTextFields
                    locale={l}
                    values={form.values}
                    onChange={form.onChange}
                    errors={form.errors}
                  />
                </Stack>
              ) : (
                <TranslationPanel
                  locale={l}
                  original={form.original}
                  draft={form.translationOf(l)}
                  onChange={(patch) => form.onTranslationChange(l, patch)}
                  errors={form.translationErrors(l)}
                  saved={saved.includes(l)}
                />
              )}
            </div>
          </TabPanel>
        ))}
      </TabPanels>
    </Tabs>
  );
}
