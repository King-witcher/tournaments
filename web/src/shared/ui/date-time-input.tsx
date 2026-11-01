"use client";

import { type ReactNode, useId, useState } from "react";

import { useIsClient } from "@/shared/hooks/use-is-client";
import { browserTimeZone, fromZoned, toZoned } from "@/shared/lib/zoned-time";
import { DatePicker } from "./date-picker";
import styles from "./date-time-input.module.scss";
import { TimePicker } from "./time-picker";

type Props = {
  /** Rótulo da data; a hora leva "Hora". */
  labelText: ReactNode;
  /** Instante em ISO (UTC), ou `null` sem data. */
  value: string | null;
  onChange: (iso: string | null) => void;
  /** Gerado quando não vem; a hora fica com `<id>-time`. */
  id?: string;
  /** Vai no campo da data (o que o `useFocusInvalid` procura). */
  name?: string;
  required?: boolean;
  disabled?: boolean;
  readOnly?: boolean;
  /** Dica embaixo da data, no lugar do formato. */
  helperText?: ReactNode;
  /** Erro calculado fora (data no passado), já traduzido: fica na data. */
  problem?: string;
  /** Erro que a API devolveu (já traduzido): fica na data e vale mais. */
  invalid?: boolean;
  invalidText?: ReactNode;
  /** Instante mais cedo que dá para marcar: os dias antes dele saem do calendário. */
  min?: string;
  /**
   * O fuso em que data e hora valem; sem ele, o do navegador, e a escolha
   * no seletor fica aqui dentro. Passe junto com `onTimeZoneChange` quando
   * outra parte da tela escreve datas nesse fuso.
   */
  timeZone?: string;
  onTimeZoneChange?: (timeZone: string) => void;
  size?: "sm" | "md" | "lg";
};

type Draft = { date: string | null; time: string };

/**
 * Data e hora como o Carbon as monta: o `DatePicker` com calendário e, ao
 * lado, o `TimePicker` (24 horas) com o seletor de fuso, que começa no do
 * navegador. O valor entra e sai em ISO (UTC); com só a data ou só a hora
 * preenchida, sai `null` e a que falta fica obrigatória. Até hidratar fica
 * vazio, porque o fuso do servidor não é o do navegador.
 */
export function DateTimeInput({
  labelText,
  value,
  onChange,
  id,
  name,
  required,
  disabled,
  readOnly,
  helperText,
  problem,
  invalid,
  invalidText,
  min,
  timeZone,
  onTimeZoneChange,
  size,
}: Props) {
  const isClient = useIsClient();
  const generated = useId();
  const dateId = id ?? generated;
  const [chosenZone, setChosenZone] = useState<string>();
  const zone = timeZone ?? chosenZone ?? (isClient ? browserTimeZone() : "UTC");

  // Data sem hora (ou o contrário) ainda não é um instante: fica aqui até
  // completar. Um valor novo que veio de fora descarta o rascunho.
  const [draft, setDraft] = useState<Draft | null>(null);
  const [seen, setSeen] = useState(value);
  if (value !== seen) {
    setSeen(value);
    setDraft(null);
  }
  const saved = isClient && value ? toZoned(value, zone) : null;
  const date = draft ? draft.date : (saved?.date ?? null);
  const time = draft ? draft.time : (saved?.time ?? "");

  const change = (next: Draft, nextZone = zone) => {
    const iso = next.date ? fromZoned(next.date, next.time, nextZone) : null;
    setDraft(iso ? null : next);
    setSeen(iso);
    onChange(iso);
  };

  return (
    <div className={styles.row}>
      <DatePicker
        id={dateId}
        name={name}
        labelText={labelText}
        value={date}
        onChange={(next) => change({ date: next, time })}
        minDate={isClient && min ? toZoned(min, zone).date : undefined}
        required={required || (date === null && time !== "")}
        disabled={disabled}
        readOnly={readOnly}
        helperText={helperText}
        problem={problem}
        invalid={invalid}
        invalidText={invalidText}
        size={size}
      />
      <TimePicker
        id={`${dateId}-time`}
        value={time}
        onChange={(next) => change({ date, time: next })}
        required={required || (date !== null && time === "")}
        disabled={disabled}
        readOnly={readOnly}
        size={size}
        timeZone={zone}
        onTimeZoneChange={(next) => {
          setChosenZone(next);
          onTimeZoneChange?.(next);
          change({ date, time }, next);
        }}
      />
    </div>
  );
}
