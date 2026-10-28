"use client";

import type { ComponentProps } from "react";

import { useIsClient } from "@/shared/hooks/use-is-client";
import { fromLocalInput, toLocalInput } from "@/shared/lib/datetime-local";
import { Input } from "./input";

type Props = Omit<ComponentProps<"input">, "type" | "value" | "onChange"> & {
  /** Instante em ISO (UTC), ou `null` sem data. */
  value: string | null;
  onChange: (iso: string | null) => void;
};

/**
 * Data e hora no fuso de quem usa; o valor entra e sai em ISO (UTC). Até
 * hidratar fica vazio, porque o fuso do servidor não é o do navegador.
 */
export function DateTimeInput({ value, onChange, ...props }: Props) {
  const isClient = useIsClient();
  return (
    <Input
      type="datetime-local"
      value={isClient && value ? toLocalInput(value) : ""}
      onChange={(e) => onChange(fromLocalInput(e.target.value))}
      {...props}
    />
  );
}
