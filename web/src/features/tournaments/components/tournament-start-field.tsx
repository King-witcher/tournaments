import { Stack } from "@carbon/react";
import { useTranslations } from "next-intl";

import { useIsClient } from "@/shared/hooks/use-is-client";
import { browserTimeZone } from "@/shared/lib/datetime-local";
import { DateTimeInput } from "@/shared/ui/date-time-input";
import { StartDateWarning } from "./start-date-warning";

type Props = {
  /** Início em ISO (UTC), ou `null` sem data. */
  value: string | null;
  onChange: (startsAt: string | null) => void;
  error?: string;
  /** Janela de check-in do formulário, para o aviso de pouco tempo. */
  checkInMinutes: number | null;
  /** O torneio já começou: não muda mais. */
  disabled: boolean;
};

/** Data e hora de início, no fuso de quem preenche. */
export function TournamentStartField({
  value,
  onChange,
  error,
  checkInMinutes,
  disabled,
}: Props) {
  const t = useTranslations("TournamentForm");
  const isClient = useIsClient();
  const timeZone = isClient ? browserTimeZone() : "…";

  return (
    <Stack gap={3}>
      <DateTimeInput
        labelText={t("startsAt")}
        helperText={t("startsAtHint", { timeZone })}
        name="startsAt"
        value={value}
        disabled={disabled}
        invalid={error !== undefined}
        invalidText={error}
        onChange={onChange}
      />
      {isClient && !disabled && (
        <StartDateWarning
          startsAt={value}
          checkInMinutes={checkInMinutes}
          now={new Date()}
        />
      )}
    </Stack>
  );
}
