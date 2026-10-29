"use client";

import { useTranslations } from "next-intl";

import { Checkbox } from "@/shared/ui/checkbox";
import { usePrintSettings } from "../hooks/use-print-settings";

type Props = {
  checked: boolean;
  onChange: (checked: boolean) => void;
  /** Torneio encerrado: não muda mais. */
  disabled: boolean;
};

/**
 * "Exigir print do placar", no formulário do torneio. Só aparece quando esta
 * API tem prints; sem eles, o valor gravado fica como está.
 */
export function PrintsRequiredField({ checked, onChange, disabled }: Props) {
  const t = useTranslations("Prints.field");
  const settings = usePrintSettings();
  if (!settings?.enabled) return null;
  return (
    <Checkbox
      labelText={t("label")}
      helperText={t("hint")}
      name="printsRequired"
      checked={checked}
      disabled={disabled}
      onChange={onChange}
    />
  );
}
