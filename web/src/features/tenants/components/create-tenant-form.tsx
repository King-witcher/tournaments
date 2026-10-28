"use client";

import { Form, Stack } from "@carbon/react";
import { useTranslations } from "next-intl";
import { useActionState, useState } from "react";

import { useRouter } from "@/i18n/navigation";
import { callApi } from "@/shared/api/call";
import { api } from "@/shared/api/client";
import { useApiErrorMessage } from "@/shared/api/use-api-error-message";
import { useIsClient } from "@/shared/hooks/use-is-client";
import { slugify } from "@/shared/lib/slugify";
import { FormActions } from "@/shared/ui/form-actions";
import { FormError } from "@/shared/ui/form-error";
import { TextInput } from "@/shared/ui/text-input";

type Failure = { code: string; message: string };

/** Erros que são de um campo: vão no `invalidText` dele, não acima dos botões. */
const NAME_ERRORS = new Set(["tenant.name_invalid"]);
const SLUG_ERRORS = new Set([
  "tenant.slug_taken",
  "tenant.slug_invalid",
  "tenant.slug_reserved",
]);

/**
 * Nome + endereço, com a prévia do link da comunidade. O endereço acompanha
 * o nome até a pessoa editá-lo.
 */
export function CreateTenantForm() {
  const t = useTranslations("NewTenant");
  const errorMessage = useApiErrorMessage();
  const router = useRouter();
  const isClient = useIsClient();
  const [name, setName] = useState("");
  const [slug, setSlug] = useState("");
  const [slugEdited, setSlugEdited] = useState(false);

  const [error, submit, pending] = useActionState<Failure | undefined>(
    async () => {
      const result = await callApi(
        api.POST("/tenants", { body: { name, slug } }),
      );
      if (!result.ok) {
        return {
          code: result.code,
          message: errorMessage(result.code, result.params),
        };
      }
      router.push(`/${result.data.slug}`);
      return undefined;
    },
    undefined,
  );

  const nameError =
    error && NAME_ERRORS.has(error.code) ? error.message : undefined;
  const slugError =
    error && SLUG_ERRORS.has(error.code) ? error.message : undefined;
  // O endereço completo só no navegador (a origem); no servidor, o caminho.
  const address = `${isClient ? window.location.origin : ""}/${slug || t("slugPlaceholder")}`;

  return (
    <Form action={submit}>
      <Stack gap={6}>
        <TextInput
          labelText={t("name")}
          name="name"
          required
          maxLength={64}
          autoComplete="off"
          value={name}
          onChange={(event) => {
            setName(event.target.value);
            if (!slugEdited) setSlug(slugify(event.target.value));
          }}
          invalid={nameError !== undefined}
          invalidText={nameError}
        />
        <TextInput
          labelText={t("slug")}
          helperText={t("slugHint", { address })}
          name="slug"
          required
          minLength={3}
          maxLength={32}
          pattern="[a-z0-9]+(-[a-z0-9]+)*"
          autoComplete="off"
          spellCheck={false}
          value={slug}
          onChange={(event) => {
            setSlugEdited(true);
            setSlug(event.target.value.toLowerCase());
          }}
          invalid={slugError !== undefined}
          invalidText={slugError}
        />
        {error && nameError === undefined && slugError === undefined && (
          <FormError>{error.message}</FormError>
        )}
        <FormActions
          submitLabel={t("submit")}
          pending={pending}
          cancelHref="/"
        />
      </Stack>
    </Form>
  );
}
