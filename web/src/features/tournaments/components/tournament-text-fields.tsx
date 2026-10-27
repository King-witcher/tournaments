import { useTranslations } from "next-intl";
import { useState } from "react";

import { routing } from "@/i18n/routing";
import { slugify } from "@/shared/lib/slugify";
import { Field } from "@/shared/ui/field";
import { Input } from "@/shared/ui/input";
import { Select } from "@/shared/ui/select";
import { Textarea } from "@/shared/ui/textarea";
import type { TournamentValues } from "../lib/form-values";

type Props = {
  values: TournamentValues;
  onChange: (patch: Partial<TournamentValues>) => void;
  /** Na criação aparecem slug e idioma, que depois não mudam. */
  isNew: boolean;
};

/** Nome, endereço, idioma, descrição e regras. O slug acompanha o nome até a pessoa editá-lo. */
export function TournamentTextFields({ values, onChange, isNew }: Props) {
  const t = useTranslations("TournamentForm");
  const tLocales = useTranslations("Locales");
  const [slugEdited, setSlugEdited] = useState(false);

  return (
    <>
      <Field label={t("name")}>
        <Input
          required
          maxLength={80}
          value={values.name}
          onChange={(e) =>
            onChange({
              name: e.target.value,
              ...(isNew && !slugEdited && { slug: slugify(e.target.value) }),
            })
          }
        />
      </Field>
      {isNew && (
        <>
          <Field label={t("slug")} hint={t("slugHint")}>
            <Input
              required
              minLength={3}
              maxLength={32}
              pattern="[a-z0-9]+(-[a-z0-9]+)*"
              value={values.slug}
              onChange={(e) => {
                setSlugEdited(true);
                onChange({ slug: e.target.value.toLowerCase() });
              }}
              className="font-mono"
            />
          </Field>
          <Field label={t("locale")} hint={t("localeHint")}>
            <Select
              value={values.locale}
              onChange={(e) =>
                onChange({
                  locale: e.target.value as TournamentValues["locale"],
                })
              }
            >
              {routing.locales.map((locale) => (
                <option key={locale} value={locale}>
                  {tLocales(locale)}
                </option>
              ))}
            </Select>
          </Field>
        </>
      )}
      <Field label={t("description")} hint={t("markdownHint")}>
        <Textarea
          rows={3}
          maxLength={2000}
          value={values.description}
          onChange={(e) => onChange({ description: e.target.value })}
        />
      </Field>
      <Field label={t("rules")} hint={t("markdownHint")}>
        <Textarea
          rows={10}
          maxLength={20000}
          value={values.rules}
          onChange={(e) => onChange({ rules: e.target.value })}
          className="font-mono text-sm"
        />
      </Field>
    </>
  );
}
