"use client";

import { NumberInput as CarbonNumberInput } from "@carbon/react";
import { useTranslations } from "next-intl";
import { type ComponentProps, type ReactNode, useId } from "react";

type CarbonProps = ComponentProps<typeof CarbonNumberInput>;

type Props = Omit<CarbonProps, "id" | "label" | "onChange" | "value"> & {
  labelText: ReactNode;
  /** `null` com o campo vazio (só com `allowEmpty`). */
  value: number | null;
  onChange: (value: number | null) => void;
  id?: string;
  /**
   * Aceita negativo (`min` abaixo de 0): o celular mostra o teclado com o
   * sinal de menos. Sem isto, o `inputMode="decimal"` e o `pattern="[0-9]*"`
   * que o Carbon põe por padrão abrem, no iOS, o teclado só de dígitos.
   */
  negative?: boolean;
};

/**
 * Sem `inputMode` nem `pattern`: o `type="number"` sozinho abre o teclado de
 * números e sinais. Nulo, e não `undefined`, que o Carbon troca pelo padrão;
 * o React não escreve atributo nulo.
 */
const SIGNED_KEYBOARD = { inputMode: null, pattern: null } as unknown as Pick<
  CarbonProps,
  "inputMode" | "pattern"
>;

/**
 * Número do Carbon com os botões − e + (rótulos traduzidos) e `onChange`
 * que já entrega o número.
 */
export function NumberInput({
  id,
  labelText,
  value,
  onChange,
  negative,
  ...props
}: Props) {
  const t = useTranslations("Common");
  const generated = useId();
  return (
    <CarbonNumberInput
      id={id ?? generated}
      label={labelText}
      value={value ?? ""}
      translateWithId={(key) =>
        key === "increment.number" ? t("increment") : t("decrement")
      }
      onChange={(_event, state) => {
        const number = Number(state.value);
        onChange(state.value === "" || Number.isNaN(number) ? null : number);
      }}
      {...props}
      {...(negative && SIGNED_KEYBOARD)}
    />
  );
}
