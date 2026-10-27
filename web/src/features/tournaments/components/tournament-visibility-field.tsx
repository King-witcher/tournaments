import { useTranslations } from "next-intl";

import type { components } from "@/shared/api/schema";
import { Field } from "@/shared/ui/field";
import { Select } from "@/shared/ui/select";

type Visibility = components["schemas"]["Visibility"];

type Props = {
  value: Visibility;
  onChange: (visibility: Visibility) => void;
  /** O torneio já encerrou. */
  disabled: boolean;
};

/** Público ou privado; muda até o torneio encerrar. */
export function TournamentVisibilityField({
  value,
  onChange,
  disabled,
}: Props) {
  const t = useTranslations("TournamentForm");

  return (
    <Field label={t("visibility")} hint={t("visibilityHint")}>
      <Select
        value={value}
        disabled={disabled}
        onChange={(e) => onChange(e.target.value as Visibility)}
      >
        <option value="public">{t("visibilityOptions.public")}</option>
        <option value="private">{t("visibilityOptions.private")}</option>
      </Select>
    </Field>
  );
}
