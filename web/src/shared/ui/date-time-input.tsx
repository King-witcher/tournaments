"use client";

import type { ComponentProps, ReactNode } from "react";

import { useIsClient } from "@/shared/hooks/use-is-client";
import { fromLocalInput, toLocalInput } from "@/shared/lib/datetime-local";
import { TextInput } from "./text-input";

type Props = Omit<
  ComponentProps<typeof TextInput>,
  "type" | "value" | "onChange" | "labelText" | "defaultValue"
> & {
  labelText: ReactNode;
  /** Instante em ISO (UTC), ou `null` sem data. */
  value: string | null;
  onChange: (iso: string | null) => void;
};

/**
 * Data e hora no fuso de quem usa (o `datetime-local` do navegador, com o
 * visual do `TextInput` do Carbon); o valor entra e sai em ISO (UTC). Até
 * hidratar fica vazio, porque o fuso do servidor não é o do navegador.
 */
export function DateTimeInput({ value, onChange, labelText, ...props }: Props) {
  const isClient = useIsClient();
  const local = isClient && value ? toLocalInput(value) : "";
  return (
    <TextInput
      {...props}
      labelText={labelText}
      type="datetime-local"
      value={local}
      onChange={(e) => onChange(fromLocalInput(e.target.value))}
    />
  );
}
