"use client";

import { TextArea as CarbonTextArea } from "@carbon/react";
import { type ComponentProps, type ReactNode, useId } from "react";

import { useFieldCheck } from "@/shared/hooks/use-field-check";
import { RequiredLabel } from "./required-label";

type Props = Omit<ComponentProps<typeof CarbonTextArea>, "id" | "labelText"> & {
  labelText: ReactNode;
  id?: string;
};

/** Texto longo (descrição, regras) do Carbon, com `id` gerado e as regras do `TextInput`. */
export function TextArea({
  id,
  labelText,
  required,
  rows = 6,
  ...props
}: Props) {
  const generated = useId();
  const check = useFieldCheck<HTMLTextAreaElement>({ ...props, required });
  return (
    <CarbonTextArea
      id={id ?? generated}
      labelText={<RequiredLabel required={required}>{labelText}</RequiredLabel>}
      required={required}
      rows={rows}
      {...props}
      ref={check.ref}
      onChange={check.onChange}
      onBlur={check.onBlur}
      invalid={check.invalid}
      invalidText={check.invalidText}
    />
  );
}
