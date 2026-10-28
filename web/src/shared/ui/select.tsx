"use client";

import { Select as CarbonSelect } from "@carbon/react";
import { type ComponentProps, type ReactNode, useId } from "react";

type Props = Omit<ComponentProps<typeof CarbonSelect>, "id" | "labelText"> & {
  /**
   * Rótulo visível. Sem ele, o select não desenha rótulo: é o modo antigo,
   * dentro de um `Field` (deprecated), que dá o `<label>` por fora.
   */
  labelText?: ReactNode;
  id?: string;
};

/**
 * `Select` do Carbon (nativo por baixo: bom no celular). As opções são
 * `<option value>` ou `SelectItem` do `@carbon/react`. Rótulo, dica e erro
 * como no `TextInput`.
 */
export function Select({ id, labelText, ...props }: Props) {
  const generated = useId();
  return (
    <CarbonSelect
      id={id ?? generated}
      labelText={labelText}
      noLabel={labelText === undefined}
      {...props}
    />
  );
}
