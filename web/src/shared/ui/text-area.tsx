"use client";

import { useTranslations } from "next-intl";
import {
  type ComponentProps,
  type ReactNode,
  type Ref,
  useId,
  useMemo,
} from "react";

import { useFieldCheck } from "@/shared/hooks/use-field-check";
import { mergeRefs } from "@/shared/lib/merge-refs";
import { TextArea as CarbonTextArea } from "./carbon/text-area";
import { RequiredLabel } from "./required-label";

type Props = Omit<
  ComponentProps<typeof CarbonTextArea>,
  "id" | "labelText" | "ref"
> & {
  labelText: ReactNode;
  id?: string;
  /** O `<textarea>`, para quem mexe no texto por fora (inserir no cursor). */
  ref?: Ref<HTMLTextAreaElement>;
};

/**
 * Texto longo (descrição, regras) do Carbon, com `id` gerado e as regras do
 * `TextInput`. Com `enableCounter`, o contador fala português ou espanhol
 * (o do Carbon diz "Character limit 280" ao leitor de tela). O `ref` chega
 * ao `<textarea>` junto com o da validação.
 */
export function TextArea({
  id,
  labelText,
  required,
  rows = 6,
  ref,
  ...props
}: Props) {
  const t = useTranslations("Common.counter");
  const generated = useId();
  const check = useFieldCheck<HTMLTextAreaElement>({ ...props, required });
  const merged = useMemo(() => mergeRefs(check.ref, ref), [check.ref, ref]);
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
      ref={merged}
      onChange={check.onChange}
      onBlur={check.onBlur}
      invalid={check.invalid}
      invalidText={check.invalidText}
    />
  );
}
