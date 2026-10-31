"use client";

import { TextArea as CarbonTextArea } from "@carbon/react";
import { useTranslations } from "next-intl";
import { type ComponentProps, type ReactNode, useId } from "react";

import { useFieldCheck } from "@/shared/hooks/use-field-check";
import { RequiredLabel } from "./required-label";

type Props = Omit<ComponentProps<typeof CarbonTextArea>, "id" | "labelText"> & {
  labelText: ReactNode;
  id?: string;
};

/**
 * Texto longo (descrição, regras) do Carbon, com `id` gerado e as regras do
 * `TextInput`. Com `enableCounter`, o contador fala português ou espanhol
 * (o do Carbon diz "Character limit 280" ao leitor de tela).
 */
export function TextArea({
  id,
  labelText,
  required,
  rows = 6,
  ...props
}: Props) {
  const t = useTranslations("Common.counter");
  const generated = useId();
  const check = useFieldCheck<HTMLTextAreaElement>({ ...props, required });
  return (
    <CarbonTextArea
      id={id ?? generated}
      labelText={<RequiredLabel required={required}>{labelText}</RequiredLabel>}
      required={required}
      rows={rows}
      translateWithId={(messageId, args) => {
        const count = args?.count ?? 0;
        const max = args?.maxCount ?? 0;
        if (messageId.endsWith(".remaining")) return t("remaining", { count });
        if (messageId.endsWith(".max-reached")) return t("maxReached");
        return t("limit", { max });
      }}
      {...props}
      ref={check.ref}
      onChange={check.onChange}
      onBlur={check.onBlur}
      invalid={check.invalid}
      invalidText={check.invalidText}
    />
  );
}
