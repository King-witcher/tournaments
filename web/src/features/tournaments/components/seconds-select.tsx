import { SelectItem } from "@carbon/react";
import type { ReactNode } from "react";

import { Select } from "@/shared/ui/select";
import { useFormatSeconds } from "../hooks/use-format-seconds";

type Props = {
  labelText: ReactNode;
  helperText: ReactNode;
  name: string;
  options: number[];
  value: number;
  error?: string;
  disabled: boolean;
  onChange: (seconds: number) => void;
};

/**
 * Um tempo do veto ou do draft, em segundos (em minutos a partir de um). Um
 * valor que veio da API fora das opções continua aparecendo.
 */
export function SecondsSelect({
  options,
  value,
  error,
  onChange,
  ...props
}: Props) {
  const formatSeconds = useFormatSeconds();
  const all = options.includes(value)
    ? options
    : [...options, value].sort((a, b) => a - b);
  return (
    <Select
      {...props}
      value={value}
      invalid={error !== undefined}
      invalidText={error}
      onChange={(e) => onChange(Number(e.target.value))}
    >
      {all.map((s) => (
        <SelectItem key={s} value={s} text={formatSeconds(s)} />
      ))}
    </Select>
  );
}
