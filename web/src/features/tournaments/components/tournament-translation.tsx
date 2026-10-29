import { FormGroup, Stack } from "@carbon/react";
import { useTranslations } from "next-intl";
import { useId } from "react";

import type { components } from "@/shared/api/schema";
import { Button } from "@/shared/ui/button";
import { Notice } from "@/shared/ui/notice";
import { TextArea } from "@/shared/ui/text-area";
import { TextInput } from "@/shared/ui/text-input";
import { controlName, type FieldErrors } from "../lib/form-errors";
import type { TranslationDraft, TranslationValues } from "../lib/form-values";

type Locale = components["schemas"]["Locale"];

type Props = {
  locale: Locale;
  /** O idioma original, para dizer o que aparece sem tradução. */
  original: Locale;
  draft: TranslationDraft;
  onChange: (patch: Partial<TranslationDraft>) => void;
  errors: FieldErrors<keyof TranslationValues>;
  /** O torneio já tem esta tradução salva: remover a apaga ao salvar. */
  saved: boolean;
};

/**
 * Um idioma que não é o original. Sem tradução, diz o que quem lê nesse
 * idioma vê e oferece "Adicionar"; com ela, nome (obrigatório), descrição e
 * regras adicionais e "Remover". O texto não se perde ao remover: adicionar
 * de novo antes de salvar o traz de volta. O foco acompanha a troca.
 */
export function TournamentTranslation({
  locale,
  original,
  draft,
  onChange,
  errors,
  saved,
}: Props) {
  const t = useTranslations("TournamentForm");
  const tLocales = useTranslations("Locales");
  const nameId = useId();
  const addId = useId();
  const language = tLocales(locale);
  const toggle = (enabled: boolean) => {
    onChange({ enabled });
    // O botão usado some: o foco vai para o que aparece no lugar dele.
    requestAnimationFrame(() =>
      document.getElementById(enabled ? nameId : addId)?.focus(),
    );
  };

  if (!draft.enabled) {
    return (
      <FormGroup legendText={language}>
        <Stack gap={5}>
          {saved && <Notice kind="warning">{t("willDelete")}</Notice>}
          <p>
            {t("translationOff", { language, original: tLocales(original) })}
          </p>
          <div>
            <Button id={addId} kind="tertiary" onClick={() => toggle(true)}>
              {t("addLanguage", { language })}
            </Button>
          </div>
        </Stack>
      </FormGroup>
    );
  }

  return (
    <FormGroup legendText={language}>
      <Stack gap={6}>
        <TextInput
          id={nameId}
          labelText={t("name")}
          helperText={t("translationHint")}
          name={controlName("name", locale)}
          required
          maxLength={80}
          lang={locale}
          value={draft.name}
          invalid={errors.name !== undefined}
          invalidText={errors.name}
          onChange={(e) => onChange({ name: e.target.value })}
        />
        <TextArea
          labelText={t("description")}
          name={controlName("description", locale)}
          rows={3}
          maxLength={2000}
          lang={locale}
          value={draft.description}
          invalid={errors.description !== undefined}
          invalidText={errors.description}
          onChange={(e) => onChange({ description: e.target.value })}
        />
        <TextArea
          labelText={t("rules")}
          name={controlName("rules", locale)}
          rows={8}
          maxLength={20000}
          lang={locale}
          value={draft.rules}
          invalid={errors.rules !== undefined}
          invalidText={errors.rules}
          onChange={(e) => onChange({ rules: e.target.value })}
        />
        <div>
          <Button kind="danger--ghost" onClick={() => toggle(false)}>
            {t("removeLanguage", { language })}
          </Button>
        </div>
      </Stack>
    </FormGroup>
  );
}
