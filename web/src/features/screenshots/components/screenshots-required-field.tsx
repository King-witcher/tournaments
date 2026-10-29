"use client";

import { useTranslations } from "next-intl";

import { Checkbox } from "@/shared/ui/checkbox";
import { useScreenshotSettings } from "../hooks/use-screenshot-settings";

type Props = {
  checked: boolean;
  onChange: (checked: boolean) => void;
  /** Torneio encerrado: não muda mais. */
  disabled: boolean;
};

/**
 * "Exigir screenshot do placar", no formulário do torneio. Só aparece quando
 * esta API tem screenshots; sem eles, o valor gravado fica como está.
 */
export function ScreenshotsRequiredField({
  checked,
  onChange,
  disabled,
}: Props) {
  const t = useTranslations("Screenshots.field");
  const settings = useScreenshotSettings();
  if (!settings?.enabled) return null;
  return (
    <Checkbox
      labelText={t("label")}
      helperText={t("hint")}
      name="screenshotsRequired"
      checked={checked}
      disabled={disabled}
      onChange={onChange}
    />
  );
}
