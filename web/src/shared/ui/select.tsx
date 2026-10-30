"use client";

import { Select as CarbonSelect } from "@carbon/react";
import { type ComponentProps, type ReactNode, useId } from "react";

import { RequiredLabel } from "./required-label";

type Props = Omit<ComponentProps<typeof CarbonSelect>, "id" | "labelText"> & {
  labelText: ReactNode;
  id?: string;
};

/**
 * `Select` do Carbon (nativo por baixo: bom no celular). As opções são
 * `<option value>` ou `SelectItem` do `@carbon/react`. Rótulo, dica e erro
 * como no `TextInput` (`required` põe a estrela no rótulo).
 */
export function Select({ id, labelText, required, ...props }: Props) {
  const generated = useId();
  return (
    <CarbonSelect
      id={id ?? generated}
      labelText={<RequiredLabel required={required}>{labelText}</RequiredLabel>}
      required={required}
      {...props}
    />
  );
}
