"use client";

import { TextInput as CarbonTextInput } from "@carbon/react";
import { type ComponentProps, type ReactNode, useId } from "react";

type Props = Omit<
  ComponentProps<typeof CarbonTextInput>,
  "id" | "labelText"
> & {
  /** Rótulo visível (obrigatório: placeholder não é rótulo). */
  labelText: ReactNode;
  /** Gerado quando não vem. */
  id?: string;
};

/**
 * Campo de texto do Carbon com `id` gerado. Rótulo em `labelText`, dica em
 * `helperText`, erro do campo em `invalid` + `invalidText` (já traduzido).
 */
export function TextInput({ id, ...props }: Props) {
  const generated = useId();
  return <CarbonTextInput id={id ?? generated} {...props} />;
}
