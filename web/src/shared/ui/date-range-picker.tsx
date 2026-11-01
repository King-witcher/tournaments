"use client";

import { DatePicker as CarbonDatePicker, DatePickerInput } from "@carbon/react";
import { useLocale, useTranslations } from "next-intl";
import {
  type ChangeEvent,
  type ComponentProps,
  type ReactNode,
  useId,
  useMemo,
} from "react";

import { dayFirstDigits } from "@/shared/lib/input-filters";
import {
  DAY_FIRST_FORMAT,
  DAY_FIRST_REGEX,
  dateOfPlain,
  fromDayFirst,
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
  // `Date`, não texto: no intervalo, o Carbon reescreve os campos com
  // `new Date(texto)`, que lê "09/11/2026" como 11 de setembro. Memorizado:
  // um array novo a cada render faria o Carbon remarcar a data (e voltar o
  // mês do calendário) a cada render.
  const dates = useMemo(
    () => [start ? dateOfPlain(start) : "", end ? dateOfPlain(end) : ""],
    [start, end],
  );
  // Os tipos do Carbon não declaram estes atributos do input, que ele repassa.
  // As barras entram sozinhas, então o teclado numérico do celular basta.
  const native = { required, inputMode: "numeric" as const, maxLength: 10 };
  const input = (point: 0 | 1) => ({
    placeholder: t("format"),
    helperText: helperText ?? t("format"),
    // O que se digita passa pelo mesmo filtro do `DatePicker`. No intervalo, o
    // Carbon não lê o que foi digitado (só no modo `single`) e o flatpickr o
    // apaga ao sair do campo: a ponta completa e válida vira valor na hora.
    onChange: (event: ChangeEvent<HTMLInputElement>) => {
      const clean = dayFirstDigits(event.target.value);
      if (clean !== event.target.value) event.target.value = clean;
      const typed = DAY_FIRST_REGEX.test(clean) ? fromDayFirst(clean) : null;
      if (typed && typed !== value[point]) {
        onChange(point === 0 ? [typed, end] : [start, typed]);
      }
    },
    ...native,
    ...props,
  });

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
        {...input(0)}
      />
      <DatePickerInput
        id={`${startId}-end`}
        labelText={
          <RequiredLabel required={required}>{endLabelText}</RequiredLabel>
        }
        {...input(1)}
      />
    </CarbonDatePicker>
  );
}
