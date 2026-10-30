import { useLocale, useTranslations } from "next-intl";
import { useState } from "react";

import { useIsClient } from "@/shared/hooks/use-is-client";
import { slugChars } from "@/shared/lib/input-filters";
import { slugify } from "@/shared/lib/slugify";
import { TextInput } from "@/shared/ui/text-input";
import type { FieldErrors } from "../lib/form-errors";
import type { TournamentValues } from "../lib/form-values";

type Props = {
  tenant: string;
  values: TournamentValues;
  onChange: (patch: Partial<TournamentValues>) => void;
  errors: FieldErrors;
  /** Na criação aparece o id, que depois não muda. */
  isNew: boolean;
};

/**
 * O nome no idioma original e, na criação, o id com a prévia do link. O id
 * acompanha o nome até a pessoa editá-lo.
 */
export function TournamentIdentityFields({
  tenant,
  values,
  onChange,
  errors,
  isNew,
}: Props) {
  const t = useTranslations("TournamentForm");
  const locale = useLocale();
  const isClient = useIsClient();
  const [slugEdited, setSlugEdited] = useState(false);
  // O domínio só no navegador: o servidor não sabe por qual endereço entraram.
  const host = isClient ? window.location.host : "";
  const address = `${host}/${locale}/${tenant}/${values.slug || "…"}`;

  return (
    <>
      <TextInput
        labelText={t("name")}
        name="name"
        required
        maxLength={80}
        lang={values.locale}
        value={values.name}
        invalid={errors.name !== undefined}
        invalidText={errors.name}
        onChange={(e) =>
          onChange({
            name: e.target.value,
            ...(isNew && !slugEdited && { slug: slugify(e.target.value) }),
          })
        }
      />
      {isNew && (
        <TextInput
          labelText={t("slug")}
          helperText={t("slugHint", { address })}
          name="slug"
          required
          minLength={3}
          maxLength={32}
          pattern="[a-z0-9]+(-[a-z0-9]+)*"
          patternMessage={t("slugInvalid")}
          filter={slugChars}
          autoCapitalize="none"
          autoComplete="off"
          spellCheck={false}
          value={values.slug}
          invalid={errors.slug !== undefined}
          invalidText={errors.slug}
          onChange={(e) => {
            setSlugEdited(true);
            onChange({ slug: e.target.value });
          }}
        />
      )}
    </>
  );
}
