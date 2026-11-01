"use client";

import { DatePicker as CarbonDatePicker, DatePickerInput } from "@carbon/react";
import { useLocale, useTranslations } from "next-intl";
import {
  type ChangeEvent,
  type ComponentProps,
  type ReactNode,
  useId,
  useLayoutEffect,
  useRef,
  useState,
} from "react";

import { useFieldCheck } from "@/shared/hooks/use-field-check";
import {
  DAY_FIRST_FORMAT,
  DAY_FIRST_PATTERN,
  DAY_FIRST_REGEX,
  fromDayFirst,
  parseDayFirst,
  plainDateOf,
  toDayFirst,
} from "@/shared/lib/plain-date";
import { RequiredLabel } from "./required-label";

type Props = Pick<
  ComponentProps<typeof DatePickerInput>,
  | "size"
  | "disabled"
  | "readOnly"
  | "hideLabel"
  | "helperText"
  | "warn"
  | "warnText"
  | "invalid"
  | "invalidText"
> & {
  labelText: ReactNode;
  /** Gerado quando não vem. */
  id?: string;
  name?: string;
  /**
   * `single` (padrão): o campo e o calendário, para a data que a pessoa
   * precisa ver perto das outras (agenda). `simple`: só o campo, para a
   * data que se sabe de cor (nascimento).
   */
  datePickerType?: "single" | "simple";
  /** "AAAA-MM-DD", ou `null` sem data. */
  value: string | null;
  onChange: (date: string | null) => void;
  /** Primeiro e último dia que o calendário deixa escolher ("AAAA-MM-DD"). */
  minDate?: string;
  maxDate?: string;
  required?: boolean;
  /** Erro calculado fora, já traduzido: aparece na hora e bloqueia o envio. */
  problem?: string;
  onBlur?: () => void;
};

/**
 * O `DatePicker` do Carbon (flatpickr por baixo) para uma data, em
 * "dd/mm/aaaa", com o calendário no idioma da página. O valor entra e sai
 * em "AAAA-MM-DD", sem fuso. A dica embaixo é o formato, como pede o
 * Carbon, a não ser que venha outra em `helperText` (o formato continua no
 * placeholder). Cobra `required`, o formato e `problem` como o `TextInput`.
 * Intervalo: `DateRangePicker`; data e hora: `DateTimeInput`.
 */
export function DatePicker({
  id,
  name,
  labelText,
  datePickerType = "single",
  value,
  onChange,
  minDate,
  maxDate,
  required,
  problem,
  invalid,
  invalidText,
  helperText,
  readOnly,
  onBlur,
  ...props
}: Props) {
  const t = useTranslations("Common.datePicker");
  const locale = useLocale();
  const generated = useId();
  const wrapper = useRef<HTMLDivElement>(null);
  // Sem calendário, "31/02/2026" passa no formato mas não é um dia: erro na
  // hora. (Com calendário, o Carbon troca o que não existe pelo dia 1.)
  const [typed, setTyped] = useState("");
  const nonexistent =
    datePickerType === "simple" &&
    DAY_FIRST_REGEX.test(typed) &&
    fromDayFirst(typed) === null;
  const check = useFieldCheck<HTMLInputElement>({
    required,
    pattern: DAY_FIRST_PATTERN,
    patternMessage: t("pattern"),
    problem: problem ?? (nonexistent ? t("nonexistent") : undefined),
    invalid,
    invalidText,
    onBlur,
  });

  // O DatePicker põe a ref dele no input: a validação o acha pelo wrapper,
  // antes dos efeitos que conferem o campo.
  useLayoutEffect(() => {
    check.ref.current = wrapper.current?.querySelector("input") ?? null;
  });
  // Os tipos do Carbon não declaram estes atributos do input, que ele repassa.
  const native = { name, required };

  return (
    <div ref={wrapper}>
      <CarbonDatePicker
        datePickerType={datePickerType}
        dateFormat={DAY_FIRST_FORMAT}
        parseDate={parseDayFirst}
        locale={locale}
        // Sem data, `undefined`: com "" o modo `simple` apagaria o que a
        // pessoa ainda está digitando.
        value={value ? toDayFirst(value) : undefined}
        minDate={minDate && toDayFirst(minDate)}
        maxDate={maxDate && toDayFirst(maxDate)}
        readOnly={readOnly}
        nextMonthAriaLabel={t("nextMonth")}
        prevMonthAriaLabel={t("previousMonth")}
        onChange={(dates) => onChange(dates[0] ? plainDateOf(dates[0]) : null)}
        // O Carbon dá ao calendário um nome em inglês ("calendar-container").
        onOpen={(_dates, _text, instance) =>
          instance.calendarContainer.setAttribute("aria-label", t("calendar"))
        }
      >
        <DatePickerInput
          id={id ?? generated}
          labelText={
            <RequiredLabel required={required}>{labelText}</RequiredLabel>
          }
          placeholder={t("format")}
          helperText={helperText ?? t("format")}
          {...native}
          {...props}
          onChange={(event: ChangeEvent<HTMLInputElement>) => {
            check.onChange(event);
            // Sem calendário, quem lê o que foi digitado é o campo.
            if (datePickerType === "simple") {
              setTyped(event.target.value);
              onChange(fromDayFirst(event.target.value));
            }
          }}
          onBlur={check.onBlur}
          invalid={check.invalid}
          invalidText={check.invalidText}
        />
      </CarbonDatePicker>
    </div>
  );
}
