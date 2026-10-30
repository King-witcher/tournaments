import { useFormatter, useTranslations } from "next-intl";

import { useIsClient } from "@/shared/hooks/use-is-client";
import { browserTimeZone } from "@/shared/lib/datetime-local";
import { DateTimeInput } from "@/shared/ui/date-time-input";
import { earliestStart, startTiming } from "../lib/form-values";

type Props = {
  /** Início em ISO (UTC), ou `null` sem data. */
  value: string | null;
  onChange: (startsAt: string | null) => void;
  error?: string;
  /** Janela de check-in do formulário: o início precisa deixá-la pela frente. */
  checkInMinutes: number | null;
  /** O torneio já começou: não muda mais. */
  disabled: boolean;
  /**
   * Na edição, o início gravado (`null` sem data): só uma data diferente dele
   * é conferida, como na API. Na criação, não vem e toda data é conferida.
   */
  saved?: string | null;
};

/**
 * Data e hora de início, no fuso de quem preenche. Data que já passou, ou
 * mais perto do que a janela de check-in, é erro do campo (e bloqueia o
 * envio). Embaixo, o fuso e, com check-in, quando ele abre.
 */
export function TournamentStartField({
  value,
  onChange,
  error,
  checkInMinutes,
  disabled,
  saved,
}: Props) {
  const t = useTranslations("TournamentForm");
  const format = useFormatter();
  const isClient = useIsClient();
  const timeZone = isClient ? browserTimeZone() : "…";
  const now = new Date();
  const checked =
    isClient && !disabled && (saved === undefined || value !== saved);
  const timing = checked ? startTiming(value, checkInMinutes, now) : null;
  const when = (date: Date) =>
    format.dateTime(date, {
      weekday: "short",
      day: "numeric",
      month: "short",
      hour: "2-digit",
      minute: "2-digit",
    });
  const opens =
    isClient && value && checkInMinutes !== null
      ? new Date(new Date(value).getTime() - checkInMinutes * 60_000)
      : null;

  return (
    <DateTimeInput
      labelText={t("startsAt")}
      helperText={
        opens
          ? t("startsAtCheckIn", { timeZone, opens: when(opens) })
          : t("startsAtHint", { timeZone })
      }
      name="startsAt"
      value={value}
      disabled={disabled}
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
      onChange={onChange}
    />
  );
}
