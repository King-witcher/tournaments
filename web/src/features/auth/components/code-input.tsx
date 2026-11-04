"use client";

import { useTranslations } from "next-intl";
import { useEffect, useId } from "react";

import { digitsOnly } from "@/shared/lib/input-filters";
import { TextInput } from "@/shared/ui/text-input";

export const CODE_LENGTH = 6;

type Props = {
  value: string;
  onChange: (code: string) => void;
  /** O erro do último envio, já traduzido. */
  error?: string;
  /** Leva o foco ao campo ao aparecer (a tela nova, no lugar do botão que sumiu). */
  focusOnMount?: boolean;
};

/**
 * O campo do código de 6 dígitos que vai por email (`name="code"`). Só aceita
 * dígitos: colar "123 456" vira "123456".
 */
export function CodeInput({ value, onChange, error, focusOnMount }: Props) {
  const t = useTranslations("Auth.code");
  const id = useId();

  useEffect(() => {
    if (focusOnMount) document.getElementById(id)?.focus();
  }, [focusOnMount, id]);

  return (
    <TextInput
      id={id}
      name="code"
      labelText={t("label")}
      required
      minLength={CODE_LENGTH}
      filter={digitsOnly}
      inputMode="numeric"
      autoComplete="one-time-code"
      value={value}
      // Sem `maxLength`: colar "123 456" passaria do tamanho antes do filtro.
      onChange={(e) => onChange(e.target.value.slice(0, CODE_LENGTH))}
      invalid={Boolean(error)}
      invalidText={error}
    />
  );
}
