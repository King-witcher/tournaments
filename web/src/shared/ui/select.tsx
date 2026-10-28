"use client";

import { Select as CarbonSelect } from "@carbon/react";
import { type ComponentProps, type ReactNode, useId } from "react";

type Props = Omit<ComponentProps<typeof CarbonSelect>, "id" | "labelText"> & {
  labelText: ReactNode;
  id?: string;
};

/**
 * `Select` do Carbon (nativo por baixo: bom no celular). As opções são
 * `<option value>` ou `SelectItem` do `@carbon/react`. Rótulo, dica e erro
 * como no `TextInput`.
 */
export function Select({ id, labelText, ...props }: Props) {
  const generated = useId();
  return <CarbonSelect id={id ?? generated} labelText={labelText} {...props} />;
}
