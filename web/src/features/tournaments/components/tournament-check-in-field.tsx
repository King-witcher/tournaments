import { SelectItem } from "@carbon/react";
import { useTranslations } from "next-intl";

import { Select } from "@/shared/ui/select";

const WINDOWS = [15, 30, 45, 60];

type Props = {
  /** Minutos antes do início; `null` sem check-in. */
  value: number | null;
  onChange: (minutes: number | null) => void;
  /** O que a API disse deste campo, já traduzido. */
  error?: string;
  /** O torneio já começou: não muda mais. */
  disabled: boolean;
};

/** Janela de check-in: quanto antes do início ele abre, ou sem check-in. */
export function TournamentCheckInField({
  value,
  onChange,
  error,
  disabled,
}: Props) {
  const t = useTranslations("TournamentForm");
  // Uma janela que veio da API fora das opções continua aparecendo.
  const windows =
    value === null || WINDOWS.includes(value)
      ? WINDOWS
      : [...WINDOWS, value].sort((a, b) => a - b);

  return (
    <Select
      labelText={t("checkIn")}
      helperText={t("checkInHint")}
      name="checkInMinutes"
      value={value ?? "off"}
      disabled={disabled}
      invalid={error !== undefined}
      invalidText={error}
      onChange={(e) =>
        onChange(e.target.value === "off" ? null : Number(e.target.value))
      }
    >
      <SelectItem value="off" text={t("checkInOff")} />
      {windows.map((minutes) => (
        <SelectItem
          key={minutes}
          value={minutes}
          text={t("checkInOption", { minutes })}
        />
      ))}
    </Select>
  );
}
