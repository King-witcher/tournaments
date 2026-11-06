import { useFormatter, useTranslations } from "next-intl";
import { memo, useState } from "react";

import { useIsClient } from "@/shared/hooks/use-is-client";
import { browserTimeZone } from "@/shared/lib/zoned-time";
import { DateTimeInput } from "@/shared/ui/date-time-input";
import type { FieldErrors } from "../lib/form-errors";
import {
  earliestStart,
  startTiming,
  type TournamentValues,
} from "../lib/form-values";
import { sameValues } from "../lib/same-values";

/** Os campos do formulário que o bloco lê. */
const KEYS = ["startsAt", "checkInMinutes"] as const;

type Props = {
  /**
   * O início (ISO em UTC, ou `null` sem data) e a janela de check-in, que o
   * início precisa deixar pela frente.
   */
  values: Pick<TournamentValues, (typeof KEYS)[number]>;
  onChange: (patch: Partial<TournamentValues>) => void;
  errors: FieldErrors;
  /** O torneio já começou: só para leitura. */
  readOnly: boolean;
  /**
   * Na edição, o início gravado (`null` sem data): só uma data diferente dele
   * é conferida, como na API. Na criação, não vem e toda data é conferida.
   */
  saved?: string | null;
};

/**
 * Data e hora de início, no fuso escolhido ao lado da hora (o do navegador,
 * de início). Data que já passou, ou mais perto do que a janela de check-in,
 * é erro do campo (e bloqueia o envio). Embaixo, com check-in, quando ele
 * abre, no mesmo fuso.
 */
export const TournamentStartField = memo(function TournamentStartField({
  values,
  onChange,
  errors,
  readOnly,
  saved,
}: Props) {
  const { startsAt: value, checkInMinutes } = values;
  const error = errors.startsAt;
  const t = useTranslations("TournamentForm");
  const format = useFormatter();
  const isClient = useIsClient();
  const [chosenZone, setChosenZone] = useState<string>();
  const timeZone = chosenZone ?? (isClient ? browserTimeZone() : "UTC");
  const now = new Date();
  const checked =
    isClient && !readOnly && (saved === undefined || value !== saved);
  const timing = checked ? startTiming(value, checkInMinutes, now) : null;
  const when = (date: Date) =>
    format.dateTime(date, {
      weekday: "short",
      day: "numeric",
      month: "short",
      hour: "2-digit",
      minute: "2-digit",
      timeZone,
    });
  const opens =
    isClient && value && checkInMinutes !== null
      ? new Date(new Date(value).getTime() - checkInMinutes * 60_000)
      : null;
  // O calendário começa hoje; um início gravado que já passou continua à
  // vista (o flatpickr apagaria uma data antes do mínimo).
  const min =
    isClient && !readOnly && (saved == null || new Date(saved) > now)
      ? now.toISOString()
      : undefined;

  return (
    <DateTimeInput
      labelText={t("startsAt")}
      helperText={
        opens ? t("startsAtCheckIn", { opens: when(opens) }) : undefined
      }
      name="startsAt"
      value={value}
      readOnly={readOnly}
      min={min}
      timeZone={timeZone}
      onTimeZoneChange={setChosenZone}
      problem={
        timing === "past"
          ? t("startError.past")
          : timing === "window"
            ? t("startError.soon", {
                from: when(earliestStart(checkInMinutes ?? 0, now)),
              })
            : undefined
      }
      invalid={error !== undefined}
      invalidText={error}
      onChange={(startsAt) => onChange({ startsAt })}
    />
  );
}, sameValues(KEYS));
