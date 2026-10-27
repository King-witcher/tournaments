"use client";

import { useTranslations } from "next-intl";
import { useActionState, useState } from "react";

import { useRouter } from "@/i18n/navigation";
import { callApi } from "@/shared/api/call";
import { api } from "@/shared/api/client";
import { useApiErrorMessage } from "@/shared/api/use-api-error-message";
import { slugify } from "@/shared/lib/slugify";
import { Alert } from "@/shared/ui/alert";
import { Button } from "@/shared/ui/button";
import { Field } from "@/shared/ui/field";
import { Input } from "@/shared/ui/input";

/** Nome + slug. O slug acompanha o nome até a pessoa editá-lo. */
export function CreateTenantForm() {
  const t = useTranslations("NewTenant");
  const errorMessage = useApiErrorMessage();
  const router = useRouter();
  const [name, setName] = useState("");
  const [slug, setSlug] = useState("");
  const [slugEdited, setSlugEdited] = useState(false);

  const [error, submit, pending] = useActionState(async () => {
    const result = await callApi(
      api.POST("/tenants", { body: { name, slug } }),
    );
    if (!result.ok) return errorMessage(result.code);
    router.push(`/${result.data.slug}`);
    return undefined;
  }, undefined);

  return (
    <form action={submit} className="flex flex-col gap-4">
      <Field label={t("name")}>
        <Input
          required
          maxLength={64}
          value={name}
          onChange={(e) => {
            setName(e.target.value);
            if (!slugEdited) setSlug(slugify(e.target.value));
          }}
        />
      </Field>
      <Field label={t("slug")} hint={t("slugHint")}>
        <Input
          required
          minLength={3}
          maxLength={32}
          pattern="[a-z0-9]+(-[a-z0-9]+)*"
          value={slug}
          onChange={(e) => {
            setSlugEdited(true);
            setSlug(e.target.value.toLowerCase());
          }}
          className="font-mono"
        />
      </Field>
      {error && <Alert>{error}</Alert>}
      <div className="flex items-center gap-4">
        <Button type="submit" disabled={pending}>
          {t("submit")}
        </Button>
        <span className="text-xs opacity-70">{t("limit")}</span>
      </div>
    </form>
  );
}
