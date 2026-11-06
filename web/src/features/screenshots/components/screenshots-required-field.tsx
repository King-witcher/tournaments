"use client";

import { useTranslations } from "next-intl";

import { Checkbox } from "@/shared/ui/checkbox";

type Props = {
  checked: boolean;
  onChange: (checked: boolean) => void;
  /** Torneio encerrado: só para leitura. */
  readOnly: boolean;
};

/**
 * "Exigir screenshot do placar", no formulário do torneio. Quem o usa só o
 * mostra quando esta API tem screenshots (`getScreenshotSettings`); sem eles, o
 * valor gravado fica como está.
 */
export function ScreenshotsRequiredField({
  checked,
  onChange,
  readOnly,
}: Props) {
  const t = useTranslations("Screenshots.field");
  return (
    <Checkbox
      labelText={t("label")}
      helperText={t("hint")}
      name="screenshotsRequired"
      checked={checked}
      readOnly={readOnly}
      onChange={onChange}
    />
  );
}
