import { useTranslations } from "next-intl";

import { Field } from "@/shared/ui/field";
import { Select } from "@/shared/ui/select";

const WINDOWS = [2, 5, 10, 15, 30, 60];

type Props = {
  value: number;
  onChange: (minutes: number) => void;
  /** Torneio encerrado: não muda mais. */
  disabled: boolean;
};

/** Janela de contestação: quanto tempo o adversário tem para responder. */
export function TournamentContestField({ value, onChange, disabled }: Props) {
  const t = useTranslations("TournamentForm");
  // Uma janela que veio da API fora das opções continua aparecendo.
  const windows = WINDOWS.includes(value)
    ? WINDOWS
    : [...WINDOWS, value].sort((a, b) => a - b);

  return (
    <fieldset disabled={disabled} className="flex flex-col gap-4">
      <legend className="mb-2 text-lg font-semibold">{t("results")}</legend>
      <Field label={t("contest")} hint={t("contestHint")}>
        <Select
          value={value}
          onChange={(e) => onChange(Number(e.target.value))}
        >
          {windows.map((minutes) => (
            <option key={minutes} value={minutes}>
              {t("contestOption", { minutes })}
            </option>
          ))}
        </Select>
      </Field>
    </fieldset>
  );
}
