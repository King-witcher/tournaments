"use client";

import { NumberInput as CarbonNumberInput } from "@carbon/react";
import { useTranslations } from "next-intl";
import { type ComponentProps, type ReactNode, useId } from "react";

type Props = Omit<
  ComponentProps<typeof CarbonNumberInput>,
  "id" | "label" | "onChange" | "value"
> & {
  labelText: ReactNode;
  /** `null` com o campo vazio (só com `allowEmpty`). */
  value: number | null;
  onChange: (value: number | null) => void;
  id?: string;
};

/**
 * Número do Carbon com os botões − e + (rótulos traduzidos) e `onChange`
 * que já entrega o número.
 */
export function NumberInput({
  id,
  labelText,
  value,
  onChange,
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
    />
  );
}
