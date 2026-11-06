"use client";

import { useTranslations } from "next-intl";

import { useIsClient } from "@/shared/hooks/use-is-client";
import { fieldId, invalidProps } from "@/shared/lib/form-result";
import { slugChars } from "@/shared/lib/input-filters";
import { TextInput } from "@/shared/ui/text-input";
import type { CreateTenantState } from "../hooks/use-create-tenant";

/** Nome e endereço da comunidade a criar, com a prévia do link. */
export function CreateTenantFields({ form }: { form: CreateTenantState }) {
  const t = useTranslations("NewTenant");
  const isClient = useIsClient();
  // O endereço completo só no navegador (a origem); no servidor, o caminho.
  const address = `${isClient ? window.location.origin : ""}/${form.slug || t("slugPlaceholder")}`;

  return (
    <>
      <TextInput
        id={fieldId(form.prefix, "name")}
        labelText={t("name")}
        name="name"
        required
        maxLength={64}
        autoComplete="off"
        value={form.name}
        onChange={(event) => form.changeName(event.target.value)}
        {...invalidProps(form.result, "name")}
      />
      <TextInput
        id={fieldId(form.prefix, "slug")}
        labelText={t("slug")}
        helperText={t("slugHint", { address })}
        name="slug"
        required
        minLength={3}
        maxLength={32}
        pattern="[a-z0-9]+(-[a-z0-9]+)*"
        patternMessage={t("slugInvalid")}
        filter={slugChars}
        autoComplete="off"
        spellCheck={false}
        value={form.slug}
        onChange={(event) => form.changeSlug(event.target.value)}
        {...invalidProps(form.result, "slug")}
      />
    </>
  );
}
