import { useLocale, useTranslations } from "next-intl";
import { memo, useState } from "react";

import { useIsClient } from "@/shared/hooks/use-is-client";
import { slugChars } from "@/shared/lib/input-filters";
import { slugify } from "@/shared/lib/slugify";
import { TextInput } from "@/shared/ui/text-input";
import type { FieldErrors } from "../lib/form-errors";
import type { TournamentValues } from "../lib/form-values";
import { sameValues } from "../lib/same-values";

/** Os campos do formulário que o bloco lê. */
const KEYS = ["name", "slug", "locale"] as const;

type Props = {
  tenant: string;
  values: Pick<TournamentValues, (typeof KEYS)[number]>;
  onChange: (patch: Partial<TournamentValues>) => void;
  errors: FieldErrors;
  /** Na criação, o endereço acompanha o nome até a pessoa editá-lo. */
  isNew: boolean;
  /** Já publicado: o endereço só para leitura (o motivo vem antes, no `Callout` da seção). */
  slugReadOnly?: boolean;
};

/**
 * O nome no idioma original e o endereço com a prévia do link. Na criação, o
 * erro do endereço aparece quando a pessoa sai do nome: um nome curto ("Q")
 * deixava os botões desabilitados sem o motivo à vista. Depois de publicar, o
 * endereço fica só para leitura, para quem procura como mudar ver que não muda.
 */
export const TournamentIdentityFields = memo(function TournamentIdentityFields({
  tenant,
  values,
  onChange,
  errors,
  isNew,
  slugReadOnly = false,
}: Props) {
  const t = useTranslations("TournamentForm");
  const locale = useLocale();
  const isClient = useIsClient();
  const [slugEdited, setSlugEdited] = useState(false);
  const [nameLeft, setNameLeft] = useState(false);
  // O domínio só no navegador: o servidor não sabe por qual endereço entraram.
  const host = isClient ? window.location.host : "";
  const address = `${host}/${locale}/${tenant}/${values.slug || t("slugPlaceholder")}`;

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
        onBlur={() => setNameLeft(true)}
        onChange={(e) =>
          onChange({
            name: e.target.value,
            ...(isNew && !slugEdited && { slug: slugify(e.target.value) }),
          })
        }
      />
      {slugReadOnly ? (
        <TextInput
          labelText={t("slug")}
          name="slug"
          readOnly
          value={values.slug}
        />
      ) : (
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
          touched={isNew ? nameLeft : undefined}
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
}, sameValues(KEYS));
