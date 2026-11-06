"use client";

import {
  type ComponentProps,
  type ReactNode,
  useEffect,
  useId,
  useRef,
} from "react";

import { Select as CarbonSelect } from "./carbon/select";
import { RequiredLabel } from "./required-label";

type Props = Omit<ComponentProps<typeof CarbonSelect>, "id" | "labelText"> & {
  labelText: ReactNode;
  id?: string;
  /**
   * Erro que o chamador calcula (já traduzido), como séries que pedem mais
   * mapas que o pool: aparece na hora e bloqueia o envio
   * (`setCustomValidity`). O erro de fora (`invalid`, o da API) vale mais.
   */
  problem?: string;
};

/**
 * `Select` do Carbon (nativo por baixo: bom no celular). As opções são
 * `<option value>` ou `SelectItem` do `@carbon/react`. Rótulo, dica e erro
 * como no `TextInput` (`required` põe a estrela no rótulo; `problem`, o erro
 * que só o chamador sabe).
 */
export function Select({
  id,
  labelText,
  required,
  problem,
  invalid,
  invalidText,
  ...props
}: Props) {
  const generated = useId();
  const ref = useRef<HTMLSelectElement>(null);
  // O `form.checkValidity()` (ver `useFormValidity`) conta o erro daqui.
  useEffect(() => {
    ref.current?.setCustomValidity(problem ?? "");
  }, [problem]);
  return (
    <CarbonSelect
      ref={ref}
      id={id ?? generated}
      labelText={<RequiredLabel required={required}>{labelText}</RequiredLabel>}
      required={required}
      invalid={invalid || problem !== undefined}
      invalidText={invalid ? invalidText : problem}
      {...props}
    />
  );
}
