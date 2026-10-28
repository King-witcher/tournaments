import { SelectItem } from "@carbon/react";
import { useLocale, useTranslations } from "next-intl";
import { useState } from "react";

import { useIsClient } from "@/shared/hooks/use-is-client";
import { slugify } from "@/shared/lib/slugify";
import { Select } from "@/shared/ui/select";
import { TextInput } from "@/shared/ui/text-input";
import type { FieldErrors } from "../lib/form-errors";
import type { TournamentValues } from "../lib/form-values";
import { TournamentFormGroup } from "./tournament-form-group";

type Props = {
  tenant: string;
  values: TournamentValues;
  onChange: (patch: Partial<TournamentValues>) => void;
  errors: FieldErrors;
  /** Na criação aparece o endereço, que depois não muda. */
  isNew: boolean;
  /** O torneio já encerrou: a visibilidade não muda mais. */
  finished: boolean;
};

/**
 * Identificação: nome, endereço (só na criação, com a prévia do link) e
 * visibilidade. O endereço acompanha o nome até a pessoa editá-lo.
 */
export function TournamentIdentityFields({
  tenant,
  values,
  onChange,
  errors,
  isNew,
  finished,
}: Props) {
  const t = useTranslations("TournamentForm");
  const locale = useLocale();
  const isClient = useIsClient();
  const [slugEdited, setSlugEdited] = useState(false);
  // O domínio só no navegador: o servidor não sabe por qual endereço entraram.
  const host = isClient ? window.location.host : "";
  const address = `${host}/${locale}/${tenant}/${values.slug || "…"}`;

  return (
    <TournamentFormGroup legend={t("identity")}>
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
          autoCapitalize="none"
          autoComplete="off"
          spellCheck={false}
          value={values.slug}
          invalid={errors.slug !== undefined}
          invalidText={errors.slug}
          onChange={(e) => {
            setSlugEdited(true);
            onChange({ slug: e.target.value.toLowerCase() });
          }}
        />
      )}
      <Select
        labelText={t("visibility")}
        helperText={t("visibilityHint")}
        name="visibility"
        value={values.visibility}
        disabled={finished}
        onChange={(e) =>
          onChange({
            visibility: e.target.value as TournamentValues["visibility"],
          })
        }
      >
        <SelectItem value="public" text={t("visibilityOptions.public")} />
        <SelectItem value="private" text={t("visibilityOptions.private")} />
      </Select>
    </TournamentFormGroup>
  );
}
