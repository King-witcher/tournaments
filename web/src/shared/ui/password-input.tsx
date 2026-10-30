"use client";

import { PasswordInput as CarbonPasswordInput } from "@carbon/react";
import { useTranslations } from "next-intl";
import { type ComponentProps, type ReactNode, useId } from "react";

import { useFieldCheck } from "@/shared/hooks/use-field-check";
import { RequiredLabel } from "./required-label";

type Props = Omit<
  ComponentProps<typeof CarbonPasswordInput>,
  "id" | "labelText" | "showPasswordLabel" | "hidePasswordLabel"
> & {
  labelText: ReactNode;
  id?: string;
  /** A senha repetida: tem de ser igual a esta ("As senhas não são iguais."). */
  sameAs?: string;
};

/**
 * Senha com o botão de mostrar/esconder do Carbon, rótulos traduzidos. Cobra
 * `required`, `minLength` e `sameAs` como o `TextInput`.
 */
export function PasswordInput({
  id,
  labelText,
  required,
  sameAs,
  ...props
}: Props) {
  const t = useTranslations("Common");
  const generated = useId();
  const check = useFieldCheck<HTMLInputElement>({ ...props, required, sameAs });
  return (
    <CarbonPasswordInput
      id={id ?? generated}
      labelText={<RequiredLabel required={required}>{labelText}</RequiredLabel>}
      required={required}
      showPasswordLabel={t("showPassword")}
      hidePasswordLabel={t("hidePassword")}
      {...props}
      ref={check.ref}
      onChange={check.onChange}
      onBlur={check.onBlur}
      invalid={check.invalid}
      invalidText={check.invalidText}
    />
  );
}
