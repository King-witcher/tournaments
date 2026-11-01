"use client";

import { DatePicker as CarbonDatePicker, DatePickerInput } from "@carbon/react";
import { useLocale, useTranslations } from "next-intl";
import { type ComponentProps, type ReactNode, useId, useMemo } from "react";

import {
  DAY_FIRST_FORMAT,
  parseDayFirst,
  plainDateOf,
  toDayFirst,
} from "@/shared/lib/plain-date";
import { RequiredLabel } from "./required-label";

type Range = [start: string | null, end: string | null];

type Props = Pick<
  ComponentProps<typeof DatePickerInput>,
  "size" | "disabled" | "warn" | "warnText" | "invalid" | "invalidText"
> & {
  /** Rótulo do começo; o Carbon pede um para cada ponta. */
  startLabelText: ReactNode;
  endLabelText: ReactNode;
  /** Gerado quando não vem; o fim fica com `<id>-end`. */
  id?: string;
  /** As duas pontas em "AAAA-MM-DD"; `null` na que falta. */
  value: Range;
  onChange: (range: Range) => void;
  /** Primeiro e último dia que o calendário deixa escolher ("AAAA-MM-DD"). */
  minDate?: string;
  maxDate?: string;
  required?: boolean;
  readOnly?: boolean;
  /** Dica embaixo de cada ponta, no lugar do formato. */
  helperText?: ReactNode;
};

/**
 * O `DatePicker` do Carbon no modo `range`: dois campos ("dd/mm/aaaa") e um
 * calendário que marca o começo e o fim, para agenda e filtro por período.
 * As pontas entram e saem em "AAAA-MM-DD"; enquanto só o começo foi
 * escolhido, o fim vem `null`. O erro da API vai em `invalid` + `invalidText`
 * e o Carbon o mostra nas duas pontas.
 */
export function DateRangePicker({
  startLabelText,
  endLabelText,
  id,
  value,
  onChange,
  minDate,
  maxDate,
  required,
  readOnly,
  helperText,
  ...props
}: Props) {
  const t = useTranslations("Common.datePicker");
  const locale = useLocale();
  const generated = useId();
  const startId = id ?? generated;
  const [start, end] = value;
  // Um array novo a cada render faria o Carbon remarcar a data (e voltar o
  // mês do calendário) a cada render.
  const dates = useMemo(
    () => [start ? toDayFirst(start) : "", end ? toDayFirst(end) : ""],
    [start, end],
  );
  // Os tipos do Carbon não declaram este atributo do input, que ele repassa.
  const native = { required };
  const input = {
    placeholder: t("format"),
    helperText: helperText ?? t("format"),
    ...native,
    ...props,
  };

  return (
    <CarbonDatePicker
      datePickerType="range"
      dateFormat={DAY_FIRST_FORMAT}
      parseDate={parseDayFirst}
      locale={locale}
      value={dates}
      minDate={minDate && toDayFirst(minDate)}
      maxDate={maxDate && toDayFirst(maxDate)}
      readOnly={readOnly}
      nextMonthAriaLabel={t("nextMonth")}
      prevMonthAriaLabel={t("previousMonth")}
      onChange={(selected) =>
        onChange([
          selected[0] ? plainDateOf(selected[0]) : null,
          selected[1] ? plainDateOf(selected[1]) : null,
        ])
      }
      // O Carbon dá ao calendário um nome em inglês ("calendar-container").
      onOpen={(_dates, _text, instance) =>
        instance.calendarContainer.setAttribute("aria-label", t("calendar"))
      }
    >
      <DatePickerInput
        id={startId}
        labelText={
          <RequiredLabel required={required}>{startLabelText}</RequiredLabel>
        }
        {...input}
      />
      <DatePickerInput
        id={`${startId}-end`}
        labelText={
          <RequiredLabel required={required}>{endLabelText}</RequiredLabel>
        }
        {...input}
      />
    </CarbonDatePicker>
  );
}
