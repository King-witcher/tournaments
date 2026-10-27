"use client";

import { useTranslations } from "next-intl";
import { useActionState, useState } from "react";
import { useRouter } from "@/i18n/navigation";
import { api } from "@/lib/api/client";
import { useApiErrorMessage } from "@/lib/api/use-api-error";
import { slugify } from "@/lib/slug";

/** Nome + slug. O slug acompanha o nome até a pessoa editá-lo. */
export function CreateTenantForm() {
  const t = useTranslations("NewTenant");
  const errorMessage = useApiErrorMessage();
  const router = useRouter();
  const [name, setName] = useState("");
  const [slug, setSlug] = useState("");
  const [slugEdited, setSlugEdited] = useState(false);

  const [error, submit, pending] = useActionState(async () => {
    const result = await api
      .POST("/tenants", { body: { name, slug } })
      .catch(() => undefined);
    if (result?.data) {
      router.push(`/${result.data.slug}`);
      return undefined;
    }
    return errorMessage(result?.error?.error);
  }, undefined);

  return (
    <form action={submit} className="flex flex-col gap-4">
      <label className="flex flex-col gap-1">
        <span className="text-sm font-medium">{t("name")}</span>
        <input
          required
          maxLength={64}
          value={name}
          onChange={(e) => {
            setName(e.target.value);
            if (!slugEdited) setSlug(slugify(e.target.value));
          }}
          className="rounded-md border border-black/20 bg-transparent px-3 py-2 dark:border-white/20"
        />
      </label>
      <label className="flex flex-col gap-1">
        <span className="text-sm font-medium">{t("slug")}</span>
        <input
          required
          minLength={3}
          maxLength={32}
          pattern="[a-z0-9]+(-[a-z0-9]+)*"
          value={slug}
          onChange={(e) => {
            setSlugEdited(true);
            setSlug(e.target.value.toLowerCase());
          }}
          className="rounded-md border border-black/20 bg-transparent px-3 py-2 font-mono dark:border-white/20"
        />
        <span className="text-xs opacity-70">{t("slugHint")}</span>
      </label>
      {error && (
        <p
          role="alert"
          className="rounded-md border border-red-500/40 bg-red-500/10 px-3 py-2 text-sm"
        >
          {error}
        </p>
      )}
      <div className="flex items-center gap-4">
        <button
          type="submit"
          disabled={pending}
          className="rounded-md bg-black px-4 py-2 text-sm font-medium text-white hover:bg-black/80 disabled:opacity-50 dark:bg-white dark:text-black dark:hover:bg-white/80"
        >
          {t("submit")}
        </button>
        <span className="text-xs opacity-70">{t("limit")}</span>
      </div>
    </form>
  );
}
