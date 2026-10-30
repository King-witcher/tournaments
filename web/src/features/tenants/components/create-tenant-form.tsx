"use client";

import { Form, Stack } from "@carbon/react";
import { useTranslations } from "next-intl";
import { useActionState, useId, useState } from "react";

import { useRouter } from "@/i18n/navigation";
import { callApi } from "@/shared/api/call";
import { api } from "@/shared/api/client";
import { useApiErrorMessage } from "@/shared/api/use-api-error-message";
import { useFocusInvalid } from "@/shared/hooks/use-focus-invalid";
import { useFormValidity } from "@/shared/hooks/use-form-validity";
import { useIsClient } from "@/shared/hooks/use-is-client";
import {
  type FormResult,
  fieldId,
  formError,
  invalidProps,
} from "@/shared/lib/form-result";
import { slugChars } from "@/shared/lib/input-filters";
import { slugify } from "@/shared/lib/slugify";
import { FormActions } from "@/shared/ui/form-actions";
import { FormError } from "@/shared/ui/form-error";
import { TextInput } from "@/shared/ui/text-input";

type Field = "name" | "slug";

/** Os erros que são de um campo: vão no `invalidText` dele, não acima dos botões. */
const FIELD_OF_ERROR: Partial<Record<string, Field>> = {
  "tenant.name_invalid": "name",
  "tenant.slug_taken": "slug",
  "tenant.slug_invalid": "slug",
  "tenant.slug_reserved": "slug",
};

/**
 * Nome + endereço, com a prévia do link da comunidade. O endereço acompanha
 * o nome até a pessoa editá-lo.
 */
export function CreateTenantForm() {
  const t = useTranslations("NewTenant");
  const errorMessage = useApiErrorMessage();
  const router = useRouter();
  const isClient = useIsClient();
  const prefix = useId();
  const { ref, canSubmit } = useFormValidity();
  const [name, setName] = useState("");
  const [slug, setSlug] = useState("");
  const [slugEdited, setSlugEdited] = useState(false);

  const [result, submit, pending] = useActionState<FormResult<Field>>(
    async () => {
      const outcome = await callApi(
        api.POST("/tenants", { body: { name, slug } }),
      );
      if (!outcome.ok) {
        return {
          error: errorMessage(outcome.code, outcome.params),
          field: FIELD_OF_ERROR[outcome.code],
        };
      }
      router.push(`/${outcome.data.slug}`);
      return {};
    },
    {},
  );
  useFocusInvalid(prefix, result);
  const general = formError(result);
  // O endereço completo só no navegador (a origem); no servidor, o caminho.
  const address = `${isClient ? window.location.origin : ""}/${slug || t("slugPlaceholder")}`;

  return (
    <Form ref={ref} action={submit}>
      <Stack gap={6}>
        <TextInput
          id={fieldId(prefix, "name")}
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
          {...invalidProps(result, "name")}
        />
        <TextInput
          id={fieldId(prefix, "slug")}
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
          value={slug}
          onChange={(event) => {
            setSlugEdited(true);
            setSlug(event.target.value);
          }}
          {...invalidProps(result, "slug")}
        />
        {general && <FormError>{general}</FormError>}
        <FormActions
          submitLabel={t("submit")}
          pending={pending}
          disabled={!canSubmit}
          cancelHref="/"
        />
      </Stack>
    </Form>
  );
}
