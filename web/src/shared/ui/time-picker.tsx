"use client";

import {
  TimePicker as CarbonTimePicker,
  SelectItem,
  TimePickerSelect,
} from "@carbon/react";
import { useTranslations } from "next-intl";
import {
  type ChangeEvent,
  type ComponentProps,
  type ReactNode,
  useId,
} from "react";

import { useFieldCheck } from "@/shared/hooks/use-field-check";
import { useIsClient } from "@/shared/hooks/use-is-client";
import { cn } from "@/shared/lib/cn";
import { timeZoneOptions } from "@/shared/lib/zoned-time";
import { RequiredLabel } from "./required-label";
import styles from "./time-picker.module.scss";

// 0:00 a 23:59, a hora com um ou dois dígitos.
const PATTERN = "([01]?[0-9]|2[0-3]):[0-5][0-9]";

type Props = Omit<
  ComponentProps<typeof CarbonTimePicker>,
  | "id"
  | "labelText"
  | "value"
  | "onChange"
  | "pattern"
  | "maxLength"
  | "placeholder"
  | "children"
  | "type"
> & {
  /** Padrão "Hora". */
  labelText?: ReactNode;
  /** Gerado quando não vem. */
  id?: string;
  /** O que está no campo ("21:30", ou o que a pessoa ainda digita). */
  value: string;
  onChange: (time: string) => void;
  required?: boolean;
  /** Erro calculado fora, já traduzido: aparece na hora e bloqueia o envio. */
  problem?: string;
  /**
   * O fuso em que a hora vale (IANA, "America/Sao_Paulo"): com ele, o
   * seletor de fuso do Carbon aparece ao lado, com todos os do navegador.
   */
  timeZone?: string;
  onTimeZoneChange?: (timeZone: string) => void;
};

/**
 * O `TimePicker` do Carbon: hora e minuto em 24 horas ("hh:mm"), como se
 * escreve em pt e es (o seletor AM/PM do Carbon só existe no formato de 12
 * horas e fica de fora). Com `timeZone`, o seletor de fuso, que o Carbon pede
 * para toda hora marcada. Cobra `required`, o formato e `problem` como o
 * `TextInput`; o erro da API vai em `invalid` + `invalidText`. Data e hora
 * juntas: `DateTimeInput`.
 */
export function TimePicker({
  id,
  labelText,
  value,
  onChange,
  required,
  problem,
  invalid,
  invalidText,
  timeZone,
  onTimeZoneChange,
  onBlur,
  disabled,
  readOnly,
  className,
  pickerClassName,
  ...props
}: Props) {
  const t = useTranslations("Common.timePicker");
  const isClient = useIsClient();
  const generated = useId();
  const inputId = id ?? generated;
  const check = useFieldCheck<HTMLInputElement>({
    required,
    pattern: PATTERN,
    patternMessage: t("pattern"),
    problem,
    invalid,
    invalidText,
    onBlur,
  });

  // Os tipos do Carbon não declaram este atributo do input, que ele repassa.
  const native = { required };

  return (
    <CarbonTimePicker
      id={inputId}
      labelText={
        <RequiredLabel required={required}>
          {labelText ?? t("label")}
        </RequiredLabel>
      }
      placeholder={t("format")}
      pattern={PATTERN}
      maxLength={5}
      inputMode="numeric"
      {...native}
      disabled={disabled}
      readOnly={readOnly}
      className={cn(styles.item, className)}
      pickerClassName={cn(styles.picker, pickerClassName)}
      {...props}
      ref={check.ref}
      value={value}
      onChange={(event) => {
        check.onChange(event);
        onChange(event.target.value);
      }}
      onBlur={check.onBlur}
      invalid={check.invalid}
      invalidText={check.invalidText}
    >
      {timeZone !== undefined && (
        <TimePickerSelect
          id={`${inputId}-zone`}
          className={styles.zone}
          aria-label={t("timeZone")}
          // Até hidratar, a lista (que depende do navegador) fica vazia.
          value={isClient ? timeZone : ""}
          disabled={disabled || readOnly}
          onChange={(event: ChangeEvent<HTMLSelectElement>) =>
            onTimeZoneChange?.(event.target.value)
          }
        >
          {isClient &&
            timeZoneOptions().map((zone) => (
              <SelectItem
                key={zone.value}
                value={zone.value}
                text={zone.label}
              />
            ))}
        </TimePickerSelect>
      )}
    </CarbonTimePicker>
  );
}
