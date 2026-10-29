import { useTranslations } from "next-intl";

import { FormSection } from "@/shared/ui/form-section";
import { TextArea } from "@/shared/ui/text-area";
import type { FieldErrors } from "../lib/form-errors";
import type { TournamentValues } from "../lib/form-values";

type Props = {
  values: TournamentValues;
  onChange: (patch: Partial<TournamentValues>) => void;
  errors: FieldErrors;
};

/**
 * As regras adicionais da organização (mapas, conduta, queda de conexão), no
 * idioma original. O formato, o check-in e o envio de resultados a aba Regras
 * do torneio já explica, por isso o campo é opcional. A legenda da seção já
 * nomeia o campo: o rótulo fica só para o leitor de tela.
 */
export function TournamentRulesSection({ values, onChange, errors }: Props) {
  const t = useTranslations("TournamentForm");

  return (
    <FormSection legend={t("rules")} description={t("rulesHint")}>
      <TextArea
        labelText={t("rules")}
        hideLabel
        name="rules"
        rows={8}
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
