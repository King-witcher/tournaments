import { useTranslations } from "next-intl";

import { useIsClient } from "@/shared/hooks/use-is-client";
import { browserTimeZone } from "@/shared/lib/datetime-local";
import { DateTimeInput } from "@/shared/ui/date-time-input";
import { Field } from "@/shared/ui/field";
import { Select } from "@/shared/ui/select";
import type { TournamentValues } from "../lib/form-values";

const WINDOWS = [15, 30, 45, 60];

type Props = {
  values: TournamentValues;
  onChange: (patch: Partial<TournamentValues>) => void;
  /** O torneio já começou: data e check-in ficam travados. */
  disabled: boolean;
  /** Publicado: a data não pode ficar vazia. */
  required: boolean;
};

/** Data e hora de início (no fuso de quem preenche) e janela de check-in. */
export function TournamentScheduleFields({
  values,
  onChange,
  disabled,
  required,
}: Props) {
  const t = useTranslations("TournamentForm");
  const isClient = useIsClient();
  const current = values.checkInMinutes;
  // Uma janela que veio da API fora das opções continua aparecendo.
  const windows =
    current === null || WINDOWS.includes(current)
      ? WINDOWS
      : [...WINDOWS, current].sort((a, b) => a - b);

  return (
    <fieldset disabled={disabled} className="flex flex-col gap-4">
      <legend className="mb-2 text-lg font-semibold">{t("schedule")}</legend>
      <Field
        label={t("startsAt")}
        hint={t("startsAtHint", {
          timeZone: isClient ? browserTimeZone() : "…",
        })}
      >
        <DateTimeInput
          value={values.startsAt}
          onChange={(startsAt) => onChange({ startsAt })}
          required={required}
          className="w-64"
        />
      </Field>
      <Field label={t("checkIn")} hint={t("checkInHint")}>
        <Select
          value={current ?? "off"}
          onChange={(e) =>
            onChange({
              checkInMinutes:
                e.target.value === "off" ? null : Number(e.target.value),
            })
          }
        >
          <option value="off">{t("checkInOff")}</option>
          {windows.map((minutes) => (
            <option key={minutes} value={minutes}>
              {t("checkInOption", { minutes })}
            </option>
          ))}
        </Select>
      </Field>
    </fieldset>
  );
}
