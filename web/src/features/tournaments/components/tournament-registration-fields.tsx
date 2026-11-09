import { useTranslations } from "next-intl";
import { memo } from "react";

import { RadioGroup } from "@/shared/ui/radio-group";
import { isTdm, type TournamentValues } from "../lib/form-values";
import { sameValues } from "../lib/same-values";
import { TournamentCheckInField } from "./tournament-check-in-field";

/** Os campos do formulário que o bloco lê. */
const KEYS = ["mode", "admission", "checkIn"] as const;

type Props = {
  values: Pick<TournamentValues, (typeof KEYS)[number]>;
  onChange: (patch: Partial<TournamentValues>) => void;
  /** O torneio já começou: o check-in só para leitura. */
  checkInReadOnly: boolean;
  /** Encerrado: o tipo de inscrição só para leitura. */
  admissionReadOnly: boolean;
};

/**
 * Inscrições: tipo de inscrição e se há check-in, os dois com
 * padrão. A data de início vem antes, no `TournamentStartField`; as vagas, no
 * Formato; a contestação e a captura de tela, em Resultados.
 */
export const TournamentRegistrationFields = memo(
  function TournamentRegistrationFields({
    values,
    onChange,
    checkInReadOnly,
    admissionReadOnly,
  }: Props) {
    const t = useTranslations("TournamentForm");

    return (
      <>
        <RadioGroup
          legendText={t("admission")}
          // "Só por link ou pela organização" já diz tudo; da aberta, falta o
          // prazo.
          helperText={
            values.admission === "open" ? t("admissionOpenHint") : undefined
          }
          name="admission"
          value={values.admission}
          readOnly={admissionReadOnly}
          options={[
            { value: "open", label: t("admissionOptions.open") },
            { value: "invite_only", label: t("admissionOptions.invite_only") },
          ]}
          onChange={(admission) => onChange({ admission })}
        />
        <TournamentCheckInField
          checked={values.checkIn}
          onChange={(checkIn) => onChange({ checkIn })}
          readOnly={checkInReadOnly}
          teams={isTdm(values)}
        />
      </>
    );
  },
  sameValues(KEYS),
);
