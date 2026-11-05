"use client";

import type { SliderProps } from "@carbon/react";
import { useTranslations } from "next-intl";
import { type ReactNode, useId } from "react";

import { Slider as CarbonSlider } from "./carbon/slider";
import { RequiredLabel } from "./required-label";

type Props = Omit<SliderProps, "id" | "labelText" | "translateWithId"> & {
  /** Rótulo visível em cima da barra. */
  labelText: NonNullable<ReactNode>;
  /** Gerado quando não vem. */
  id?: string;
};

/**
 * `Slider` do Carbon: um valor numa faixa conhecida (`min` a `max`), com a
 * barra para arrastar e o campo numérico ao lado para digitar o exato
 * (`hideTextInput` tira o campo e mostra o valor na dica da alça). Com
 * `unstable_valueUpper`, vira faixa com duas alças (de `value` a
 * `unstable_valueUpper`; `onChange` entrega os dois). Valor preciso sem faixa
 * à vista: `NumberInput`. O número digitado fora da faixa ou do `step` é
 * corrigido, com o aviso traduzido; `required` põe a estrela no rótulo; nas
 * duas alças, o leitor de tela lê "valor mínimo" e "valor máximo" (mude com
 * `ariaLabelInput` e `unstable_ariaLabelInputUpper`). Gera o `id`.
 */
export function Slider({
  id,
  labelText,
  required,
  ariaLabelInput,
  unstable_ariaLabelInputUpper,
  ...props
}: Props) {
  const t = useTranslations("Common.slider");
  const generated = useId();
  const range = props.unstable_valueUpper !== undefined;
  return (
    <CarbonSlider
      id={id ?? generated}
      labelText={<RequiredLabel required={required}>{labelText}</RequiredLabel>}
      required={required}
      // Com uma alça, o rótulo visível já dá o nome ao campo.
      ariaLabelInput={ariaLabelInput ?? (range ? t("lower") : undefined)}
      unstable_ariaLabelInputUpper={unstable_ariaLabelInputUpper ?? t("upper")}
      translateWithId={(_id, args) =>
        t("corrected", { value: args?.correctedValue ?? "" })
      }
      {...props}
    />
  );
}
