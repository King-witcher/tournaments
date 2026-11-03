import { useLocale, useTranslations } from "next-intl";
import { memo, useId, useState } from "react";

import { useIsClient } from "@/shared/hooks/use-is-client";
import { slugChars } from "@/shared/lib/input-filters";
import { slugify } from "@/shared/lib/slugify";
import { TextInput } from "@/shared/ui/text-input";
import type { FieldErrors } from "../lib/form-errors";
import type { TournamentValues } from "../lib/form-values";
import { sameValues } from "../lib/same-values";
import styles from "./tournament-identity-fields.module.scss";

/** Os campos do formulário que o bloco lê. */
const KEYS = ["name", "slug", "locale"] as const;

type Props = {
  tenant: string;
  values: Pick<TournamentValues, (typeof KEYS)[number]>;
  onChange: (patch: Partial<TournamentValues>) => void;
  errors: FieldErrors;
  /** Na criação, o endereço acompanha o nome até a pessoa editá-lo. */
  isNew: boolean;
  /** Já publicado: o endereço aparece travado, com o motivo embaixo. */
  slugLocked?: boolean;
};

/**
 * O nome no idioma original e o endereço com a prévia do link. Na criação, o
 * erro do endereço aparece quando a pessoa sai do nome: um nome curto ("Q")
 * deixava os botões desabilitados sem o motivo à vista. Depois de publicar, o
 * campo fica desabilitado, para quem procura como mudar ver que não muda; o
 * motivo fica numa linha à parte, porque a dica de um campo desabilitado sai no
 * cinza do Carbon, sem o contraste de um texto para ler.
 */
export const TournamentIdentityFields = memo(function TournamentIdentityFields({
  tenant,
  values,
  onChange,
  errors,
  isNew,
  slugLocked = false,
}: Props) {
  const t = useTranslations("TournamentForm");
  const locale = useLocale();
  const isClient = useIsClient();
  const noteId = useId();
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
      {slugLocked ? (
        <div>
          <TextInput
            labelText={t("slug")}
            name="slug"
            disabled
            value={values.slug}
            aria-describedby={noteId}
          />
          <p id={noteId} className={styles.note}>
            {t("slugLocked")}
          </p>
        </div>
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
