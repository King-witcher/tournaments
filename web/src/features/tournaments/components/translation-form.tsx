"use client";

import { useTranslations } from "next-intl";
import { useActionState, useState } from "react";

import { useRouter } from "@/i18n/navigation";
import { callApi } from "@/shared/api/call";
import { api } from "@/shared/api/client";
import type { components } from "@/shared/api/schema";
import { useApiErrorMessage } from "@/shared/api/use-api-error-message";
import { Alert } from "@/shared/ui/alert";
import { Button } from "@/shared/ui/button";
import { Field } from "@/shared/ui/field";
import { Input } from "@/shared/ui/input";
import { Textarea } from "@/shared/ui/textarea";
import { DeleteTranslationButton } from "./delete-translation-button";

type Schemas = components["schemas"];
type Props = {
  tenant: string;
  tournament: string;
  locale: Schemas["Locale"];
  translation?: Schemas["TournamentTranslation"];
};
type State = { saved: boolean; error?: string };

export function TranslationForm({
  tenant,
  tournament,
  locale,
  translation,
}: Props) {
  const t = useTranslations("Tournament.translation");
  const tForm = useTranslations("TournamentForm");
  const errorMessage = useApiErrorMessage();
  const router = useRouter();
  const [name, setName] = useState(translation?.name ?? "");
  const [description, setDescription] = useState(
    translation?.description ?? "",
  );
  const [rules, setRules] = useState(translation?.rules ?? "");

  const [state, submit, pending] = useActionState<State>(
    async () => {
      const result = await callApi(
        api.PUT(
          "/tenants/{slug}/tournaments/{tournament}/translations/{locale}",
          {
            params: { path: { slug: tenant, tournament, locale } },
            body: { name, description, rules },
          },
        ),
      );
      if (!result.ok) {
        return {
          saved: false,
          error: errorMessage(result.code, result.params),
        };
      }
      router.refresh();
      return { saved: true };
    },
    { saved: false },
  );

  return (
    <form action={submit} className="flex flex-col gap-4">
      <Field label={tForm("name")}>
        <Input
          required
          maxLength={80}
          value={name}
          onChange={(e) => setName(e.target.value)}
        />
      </Field>
      <Field label={tForm("description")} hint={tForm("markdownHint")}>
        <Textarea
          rows={3}
          maxLength={2000}
          value={description}
          onChange={(e) => setDescription(e.target.value)}
        />
      </Field>
      <Field label={tForm("rules")} hint={tForm("markdownHint")}>
        <Textarea
          rows={10}
          maxLength={20000}
          value={rules}
          onChange={(e) => setRules(e.target.value)}
          className="font-mono text-sm"
        />
      </Field>
      {state.error && <Alert>{state.error}</Alert>}
      <div className="flex flex-wrap items-start gap-3">
        <Button type="submit" variant="outline" disabled={pending}>
          {t("save")}
        </Button>
        {translation && (
          <DeleteTranslationButton
            tenant={tenant}
            tournament={tournament}
            locale={locale}
          />
        )}
        {state.saved && !pending && (
          <p className="py-2 text-sm opacity-70">{t("saved")}</p>
        )}
      </div>
    </form>
  );
}
