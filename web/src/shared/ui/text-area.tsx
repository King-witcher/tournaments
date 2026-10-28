"use client";

import { TextArea as CarbonTextArea } from "@carbon/react";
import { type ComponentProps, type ReactNode, useId } from "react";

type Props = Omit<ComponentProps<typeof CarbonTextArea>, "id" | "labelText"> & {
  labelText: ReactNode;
  id?: string;
};

/** Texto longo (descrição, regras) do Carbon, com `id` gerado. */
export function TextArea({ id, rows = 6, ...props }: Props) {
  const generated = useId();
  return <CarbonTextArea id={id ?? generated} rows={rows} {...props} />;
}
