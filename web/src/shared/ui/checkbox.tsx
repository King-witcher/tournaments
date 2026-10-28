"use client";

import { Checkbox as CarbonCheckbox } from "@carbon/react";
import { type ComponentProps, type ReactNode, useId } from "react";

type Props = Omit<
  ComponentProps<typeof CarbonCheckbox>,
  "id" | "labelText" | "onChange" | "checked"
> & {
  labelText: NonNullable<ReactNode>;
  checked: boolean;
  onChange: (checked: boolean) => void;
  id?: string;
};

/** Caixa de seleção do Carbon, com `onChange` que já entrega o booleano. */
export function Checkbox({ id, checked, onChange, ...props }: Props) {
  const generated = useId();
  return (
    <CarbonCheckbox
      id={id ?? generated}
      checked={checked}
      onChange={(_event, { checked }) => onChange(checked)}
      {...props}
    />
  );
}
