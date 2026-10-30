"use client";

import { NumberInput as CarbonNumberInput } from "@carbon/react";
import { useTranslations } from "next-intl";
import {
  type ComponentProps,
  type KeyboardEvent,
  type ReactNode,
  useId,
} from "react";

import { RequiredLabel } from "./required-label";

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
 * que já entrega o número. `min` e `max` ficam vermelhos na hora, com a
 * faixa no erro ("Entre 1 e 10."); `invalidText` troca essa mensagem. Inteiro
 * (sem `step` decimal): letra, `e`, `+`, vírgula e ponto não entram, e o
 * `-` só com `negative`. `required` põe a estrela no rótulo.
 */
export function NumberInput({
  id,
  labelText,
  value,
  onChange,
  negative,
  required,
  invalidText,
  onKeyDown,
  ...props
}: Props) {
  const t = useTranslations("Common");
  const generated = useId();
  const { min, max } = props;
  const signed = negative || (typeof min === "number" && min < 0);
  const integer = Number.isInteger(props.step ?? 1);
  const rangeText =
    min !== undefined && max !== undefined
      ? t("validation.range", { min, max })
      : min !== undefined
        ? t("validation.atLeast", { min })
        : max !== undefined
          ? t("validation.atMost", { max })
          : undefined;
  const blocked = (key: string) =>
    key === "e" ||
    key === "E" ||
    key === "+" ||
    (key === "-" && !signed) ||
    (integer && (key === "." || key === ","));
  return (
    <CarbonNumberInput
      id={id ?? generated}
      label={<RequiredLabel required={required}>{labelText}</RequiredLabel>}
      value={value ?? ""}
      required={required}
      invalidText={invalidText ?? rangeText}
      translateWithId={(key) =>
        key === "increment.number" ? t("increment") : t("decrement")
      }
      onKeyDown={(event: KeyboardEvent<HTMLInputElement>) => {
        if (blocked(event.key)) event.preventDefault();
        onKeyDown?.(event);
      }}
      onChange={(_event, state) => {
        const number = Number(state.value);
        onChange(state.value === "" || Number.isNaN(number) ? null : number);
      }}
      {...props}
      {...(negative && SIGNED_KEYBOARD)}
    />
  );
}
