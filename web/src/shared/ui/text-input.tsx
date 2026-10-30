"use client";

import { TextInput as CarbonTextInput } from "@carbon/react";
import { type ComponentProps, type ReactNode, useId } from "react";

import { useFieldCheck } from "@/shared/hooks/use-field-check";
import { RequiredLabel } from "./required-label";

type Props = Omit<
  ComponentProps<typeof CarbonTextInput>,
  "id" | "labelText"
> & {
  /** Rótulo visível (obrigatório: placeholder não é rótulo). */
  labelText: ReactNode;
  /** Gerado quando não vem. */
  id?: string;
  /** Tira do texto o que o campo não aceita, ao digitar (ver `shared/lib/input-filters.ts`). */
  filter?: (value: string) => string;
  /** O erro do `pattern`, curto e já traduzido ("Use de 3 a 20 letras, números, _ . ou -."). */
  patternMessage?: ReactNode;
  /** Erro calculado fora, já traduzido: aparece na hora e bloqueia o envio (ver `useFieldCheck`). */
  problem?: string;
};

/**
 * Campo de texto do Carbon com `id` gerado. Rótulo em `labelText`; dica em
 * `helperText`, que diz o que o campo é, nunca o limite. O limite está nas
 * props nativas e o campo o cobra: `required` põe a estrela no rótulo;
 * `minLength`, `maxLength` (não deixa passar), `pattern` (+ `patternMessage`) e
 * `type="email"` ficam vermelhos com a mensagem curta depois que a pessoa sai
 * do campo; `filter` não deixa entrar o caractere recusado. O erro que a API
 * devolveu vai em `invalid` + `invalidText` (já traduzido) e vale mais que os
 * daqui.
 */
export function TextInput({
  id,
  labelText,
  required,
  filter,
  patternMessage,
  problem,
  ...props
}: Props) {
  const generated = useId();
  const check = useFieldCheck<HTMLInputElement>({
    ...props,
    required,
    filter,
    patternMessage,
    problem,
  });
  return (
    <CarbonTextInput
      id={id ?? generated}
      labelText={<RequiredLabel required={required}>{labelText}</RequiredLabel>}
      required={required}
      {...props}
      ref={check.ref}
      onChange={check.onChange}
      onBlur={check.onBlur}
      invalid={check.invalid}
      invalidText={check.invalidText}
    />
  );
}
