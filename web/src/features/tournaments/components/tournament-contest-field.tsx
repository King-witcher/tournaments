import { SelectItem } from "@carbon/react";
import { useTranslations } from "next-intl";

import { Select } from "@/shared/ui/select";

const WINDOWS = [2, 5, 10, 15, 30, 60];

type Props = {
  value: number;
  onChange: (minutes: number) => void;
  /** O que a API disse deste campo, já traduzido. */
  error?: string;
  /** Torneio encerrado: não muda mais. */
  disabled: boolean;
};

/** Janela de contestação: quanto tempo o adversário tem para responder. */
export function TournamentContestField({
  value,
  onChange,
  error,
  disabled,
}: Props) {
  const t = useTranslations("TournamentForm");
  // Uma janela que veio da API fora das opções continua aparecendo.
  const windows = WINDOWS.includes(value)
    ? WINDOWS
    : [...WINDOWS, value].sort((a, b) => a - b);

  return (
    <Select
      labelText={t("contest")}
      helperText={t("contestHint")}
      name="contestMinutes"
      value={value}
      disabled={disabled}
      invalid={error !== undefined}
      invalidText={error}
      onChange={(e) => onChange(Number(e.target.value))}
    >
      {windows.map((minutes) => (
        <SelectItem
          key={minutes}
          value={minutes}
          text={t("contestOption", { minutes })}
        />
      ))}
    </Select>
  );
}
