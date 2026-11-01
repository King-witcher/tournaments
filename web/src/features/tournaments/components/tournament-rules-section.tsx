import { useTranslations } from "next-intl";
import { memo } from "react";

import { FormSection } from "@/shared/ui/form-section";
import { TextArea } from "@/shared/ui/text-area";
import type { FieldErrors } from "../lib/form-errors";
import type { TournamentValues } from "../lib/form-values";
import { sameValues } from "../lib/same-values";
import styles from "./tournament-rules-section.module.scss";

/** Os campos do formulário que o bloco lê. */
const KEYS = ["rules", "locale"] as const;

type Props = {
  values: Pick<TournamentValues, (typeof KEYS)[number]>;
  onChange: (patch: Partial<TournamentValues>) => void;
  errors: FieldErrors;
  /** Dentro de um item de acordeão, que já dá o título: sem a seção em volta. */
  bare?: boolean;
};

/**
 * As regras adicionais da organização (mapas, conduta, queda de conexão), no
 * idioma original. O formato, o check-in e o envio de resultados a aba Regras
 * do torneio já explica, por isso o campo é opcional. A legenda da seção (ou
 * o título do acordeão) já nomeia o campo: o rótulo fica só para o leitor de
 * tela.
 */
export const TournamentRulesSection = memo(function TournamentRulesSection({
  values,
  onChange,
  errors,
  bare = false,
}: Props) {
  const t = useTranslations("TournamentForm");
  const field = (
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
  );

  if (bare) {
    return (
      <>
        <p className={styles.hint}>{t("rulesHint")}</p>
        {field}
      </>
    );
  }
  return (
    <FormSection legend={t("rules")} description={t("rulesHint")}>
      {field}
    </FormSection>
  );
}, sameValues(KEYS));
