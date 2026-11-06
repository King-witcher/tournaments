import { useTranslations } from "next-intl";

import { useFormatMinutes } from "@/shared/hooks/use-format-minutes";
import { SelectItem } from "@/shared/ui/carbon/select";
import { Select } from "@/shared/ui/select";

const WINDOWS = [2, 5, 10, 15, 30, 60];

type Props = {
  value: number;
  onChange: (minutes: number) => void;
  /** O que a API disse deste campo, já traduzido. */
  error?: string;
  /** Torneio encerrado: só para leitura. */
  readOnly: boolean;
};

/** Janela de contestação: quanto tempo o adversário tem para responder. */
export function TournamentContestField({
  value,
  onChange,
  error,
  readOnly,
}: Props) {
  const t = useTranslations("TournamentForm");
  const formatMinutes = useFormatMinutes();
  // Uma janela que veio da API fora das opções continua aparecendo.
  const windows = WINDOWS.includes(value)
    ? WINDOWS
    : [...WINDOWS, value].sort((a, b) => a - b);
  // Acima de uma hora, em horas, como no resto do site ("24 h").
  const label = (minutes: number) =>
    minutes > 60 ? formatMinutes(minutes) : t("contestOption", { minutes });

  return (
    <Select
      labelText={t("contest")}
      helperText={t("contestHint")}
      name="contestMinutes"
      value={value}
      readOnly={readOnly}
      invalid={error !== undefined}
      invalidText={error}
      onChange={(e) => onChange(Number(e.target.value))}
    >
      {windows.map((minutes) => (
        <SelectItem key={minutes} value={minutes} text={label(minutes)} />
      ))}
    </Select>
  );
}
