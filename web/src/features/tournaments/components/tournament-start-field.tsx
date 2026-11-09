import { useTranslations } from "next-intl";
import { memo, useState } from "react";

import { useIsClient } from "@/shared/hooks/use-is-client";
import { browserTimeZone } from "@/shared/lib/zoned-time";
import { DateTimeInput } from "@/shared/ui/date-time-input";
import type { FieldErrors } from "../lib/form-errors";
import type { TournamentValues } from "../lib/form-values";
import { sameValues } from "../lib/same-values";

/** Os campos do formulário que o bloco lê. */
const KEYS = ["startsAt"] as const;

type Props = {
  /** O início (ISO em UTC, ou `null` sem data). */
  values: Pick<TournamentValues, (typeof KEYS)[number]>;
  onChange: (patch: Partial<TournamentValues>) => void;
  errors: FieldErrors;
  /** O torneio já começou: só para leitura. */
  readOnly: boolean;
};

/**
 * Data e hora de início, no fuso escolhido ao lado da hora (o do navegador,
 * de início). Só informação: aceita data passada, e nada acontece por ela.
 */
export const TournamentStartField = memo(function TournamentStartField({
  values,
  onChange,
  errors,
  readOnly,
}: Props) {
  const { startsAt: value } = values;
  const error = errors.startsAt;
  const t = useTranslations("TournamentForm");
  const isClient = useIsClient();
  const [chosenZone, setChosenZone] = useState<string>();
  const timeZone = chosenZone ?? (isClient ? browserTimeZone() : "UTC");

  return (
    <DateTimeInput
      labelText={t("startsAt")}
      name="startsAt"
      value={value}
      readOnly={readOnly}
      timeZone={timeZone}
      onTimeZoneChange={setChosenZone}
      invalid={error !== undefined}
      invalidText={error}
      onChange={(startsAt) => onChange({ startsAt })}
    />
  );
}, sameValues(KEYS));
