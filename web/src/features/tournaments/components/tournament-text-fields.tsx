import { SelectItem } from "@carbon/react";
import { useTranslations } from "next-intl";

import { routing } from "@/i18n/routing";
import { FormSection } from "@/shared/ui/form-section";
import { Select } from "@/shared/ui/select";
import { TextArea } from "@/shared/ui/text-area";
import type { FieldErrors } from "../lib/form-errors";
import type { TournamentValues } from "../lib/form-values";

type Props = {
  values: TournamentValues;
  onChange: (patch: Partial<TournamentValues>) => void;
  errors: FieldErrors;
  /** Na criação aparece o idioma, que depois não muda. */
  isNew: boolean;
};

/** Textos no idioma original: o idioma (só na criação), a descrição e as regras. */
export function TournamentTextFields({
  values,
  onChange,
  errors,
  isNew,
}: Props) {
  const t = useTranslations("TournamentForm");
  const tLocales = useTranslations("Locales");

  return (
    <FormSection
      legend={
        isNew ? t("texts") : t("textsIn", { language: tLocales(values.locale) })
      }
    >
      {isNew && (
        <Select
          labelText={t("locale")}
          helperText={t("localeHint")}
          name="locale"
          value={values.locale}
          onChange={(e) =>
            onChange({ locale: e.target.value as TournamentValues["locale"] })
          }
        >
          {routing.locales.map((locale) => (
            <SelectItem key={locale} value={locale} text={tLocales(locale)} />
          ))}
        </Select>
      )}
      <TextArea
        labelText={t("description")}
        helperText={t("markdownHint")}
        name="description"
        rows={4}
        maxLength={2000}
        lang={values.locale}
        value={values.description}
        invalid={errors.description !== undefined}
        invalidText={errors.description}
        onChange={(e) => onChange({ description: e.target.value })}
      />
      <TextArea
        labelText={t("rules")}
        helperText={t("markdownHint")}
        name="rules"
        rows={10}
        maxLength={20000}
        lang={values.locale}
        value={values.rules}
        invalid={errors.rules !== undefined}
        invalidText={errors.rules}
        onChange={(e) => onChange({ rules: e.target.value })}
      />
    </FormSection>
  );
}
