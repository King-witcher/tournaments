"use client";

import { Toggle as CarbonToggle } from "@carbon/react";
import { useTranslations } from "next-intl";
import { useId } from "react";

type Props = {
  labelText: string;
  toggled: boolean;
  onToggle: (toggled: boolean) => void;
  id?: string;
};

/**
 * Interruptor do Carbon para uma preferência que vale na hora (sem
 * "Salvar"), com o "Ligado"/"Desligado" traduzido ao lado e `id` gerado.
 */
export function Toggle({ id, labelText, toggled, onToggle }: Props) {
  const t = useTranslations("Common.toggle");
  const generated = useId();
  return (
    <CarbonToggle
      id={id ?? generated}
      labelText={labelText}
      labelA={t("off")}
      labelB={t("on")}
      toggled={toggled}
      onToggle={onToggle}
    />
  );
}
