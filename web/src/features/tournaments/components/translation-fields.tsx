import { useTranslations } from "next-intl";
import { useId } from "react";

import type { components } from "@/shared/api/schema";
import { FormSection } from "@/shared/ui/form-section";
import { TextArea } from "@/shared/ui/text-area";
import { TextInput } from "@/shared/ui/text-input";
import { controlName, type FieldErrors } from "../lib/form-errors";
import { type TranslationValues, translationChanged } from "../lib/form-values";
import { DeleteTranslationButton } from "./delete-translation-button";

type Props = {
  tenant: string;
  tournament: components["schemas"]["Tournament"];
  locale: components["schemas"]["Locale"];
  values: TranslationValues;
  onChange: (patch: Partial<TranslationValues>) => void;
  errors: FieldErrors<keyof TranslationValues>;
  /** Removida: os campos voltam a ficar vazios. */
  onRemoved: () => void;
};

/**
 * A tradução opcional para um idioma diferente do original, salva junto com
 * o resto do formulário; remover é na hora, com confirmação.
 */
export function TranslationFields({
  tenant,
  tournament,
  locale,
  values,
  onChange,
  errors,
  onRemoved,
}: Props) {
  const t = useTranslations("TournamentForm");
  const tLocales = useTranslations("Locales");
  const nameId = useId();
  // Já existe: dá para removê-la. Existe ou mudou: vai ser gravada no
  // "Salvar", então o nome é obrigatório.
  const saved = tournament.translations.some((x) => x.locale === locale);
  const required = saved || translationChanged(tournament, locale, values);

  return (
    <FormSection
      legend={t("translation.title", { language: tLocales(locale) })}
      description={t("translation.hint")}
    >
      <TextInput
        id={nameId}
        labelText={t("name")}
        name={controlName("name", locale)}
        required={required}
        maxLength={80}
        lang={locale}
        value={values.name}
        invalid={errors.name !== undefined}
        invalidText={errors.name}
        onChange={(e) => onChange({ name: e.target.value })}
      />
      <TextArea
        labelText={t("description")}
        helperText={t("markdownHint")}
        name={controlName("description", locale)}
        rows={4}
        maxLength={2000}
        lang={locale}
        value={values.description}
        invalid={errors.description !== undefined}
        invalidText={errors.description}
        onChange={(e) => onChange({ description: e.target.value })}
      />
      <TextArea
        labelText={t("rules")}
        helperText={t("markdownHint")}
        name={controlName("rules", locale)}
        rows={10}
        maxLength={20000}
        lang={locale}
        value={values.rules}
        invalid={errors.rules !== undefined}
        invalidText={errors.rules}
        onChange={(e) => onChange({ rules: e.target.value })}
      />
      {saved && (
        <DeleteTranslationButton
          tenant={tenant}
          tournament={tournament.slug}
          locale={locale}
          focusAfter={nameId}
          onRemoved={onRemoved}
        />
      )}
    </FormSection>
  );
}
