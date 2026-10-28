import { useTranslations } from "next-intl";

import type { components } from "@/shared/api/schema";
import { TextArea } from "@/shared/ui/text-area";
import { TextInput } from "@/shared/ui/text-input";
import { controlName, type FieldErrors } from "../lib/form-errors";
import type { TranslationValues } from "../lib/form-values";

type Props = {
  /** O idioma dos textos (o `lang` dos campos). */
  locale: components["schemas"]["Locale"];
  values: TranslationValues;
  onChange: (patch: Partial<TranslationValues>) => void;
  errors: FieldErrors<keyof TranslationValues>;
  /**
   * Os textos são de uma tradução: entra o nome (o original fica no topo do
   * formulário) e os `name` dos controles levam o idioma.
   */
  translation?: boolean;
  /** O `id` do campo do nome, para receber o foco ao ligar a tradução. */
  nameId?: string;
};

/**
 * Descrição e regras de um idioma, em texto simples (as quebras de linha
 * ficam), e o nome quando é uma tradução.
 */
export function TournamentTextFields({
  locale,
  values,
  onChange,
  errors,
  translation = false,
  nameId,
}: Props) {
  const t = useTranslations("TournamentForm");
  const name = (field: keyof TranslationValues) =>
    translation ? controlName(field, locale) : field;

  return (
    <>
      {translation && (
        <TextInput
          id={nameId}
          labelText={t("name")}
          name={name("name")}
          required
          maxLength={80}
          lang={locale}
          value={values.name}
          invalid={errors.name !== undefined}
          invalidText={errors.name}
          onChange={(e) => onChange({ name: e.target.value })}
        />
      )}
      <TextArea
        labelText={t("description")}
        helperText={t("plainTextHint")}
        name={name("description")}
        rows={3}
        maxLength={2000}
        lang={locale}
        value={values.description}
        invalid={errors.description !== undefined}
        invalidText={errors.description}
        onChange={(e) => onChange({ description: e.target.value })}
      />
      <TextArea
        labelText={t("rules")}
        helperText={t("plainTextHint")}
        name={name("rules")}
        rows={8}
        maxLength={20000}
        lang={locale}
        value={values.rules}
        invalid={errors.rules !== undefined}
        invalidText={errors.rules}
        onChange={(e) => onChange({ rules: e.target.value })}
      />
    </>
  );
}
