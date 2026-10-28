"use client";

import { PasswordInput as CarbonPasswordInput } from "@carbon/react";
import { useTranslations } from "next-intl";
import { type ComponentProps, type ReactNode, useId } from "react";

type Props = Omit<
  ComponentProps<typeof CarbonPasswordInput>,
  "id" | "labelText" | "showPasswordLabel" | "hidePasswordLabel"
> & {
  labelText: ReactNode;
  id?: string;
};

/** Senha com o botão de mostrar/esconder do Carbon, rótulos traduzidos. */
export function PasswordInput({ id, ...props }: Props) {
  const t = useTranslations("Common");
  const generated = useId();
  return (
    <CarbonPasswordInput
      id={id ?? generated}
      showPasswordLabel={t("showPassword")}
      hidePasswordLabel={t("hidePassword")}
      {...props}
    />
  );
}
