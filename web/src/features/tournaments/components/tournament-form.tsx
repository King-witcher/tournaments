"use client";

import { useLocale, useTranslations } from "next-intl";
import { useActionState, useState } from "react";

import { useRouter } from "@/i18n/navigation";
import { callApi } from "@/shared/api/call";
import { api } from "@/shared/api/client";
import type { components } from "@/shared/api/schema";
import { useApiErrorMessage } from "@/shared/api/use-api-error-message";
import { Alert } from "@/shared/ui/alert";
import { Button } from "@/shared/ui/button";
import {
  defaultValues,
  setupBody,
  type TournamentValues,
  valuesFrom,
} from "../lib/form-values";
import { setupEditable } from "../lib/status";
import { TournamentSetupFields } from "./tournament-setup-fields";
import { TournamentTextFields } from "./tournament-text-fields";

type Props = {
  tenant: string;
  /** Sem torneio, cria um rascunho; com torneio, edita. */
  tournament?: components["schemas"]["Tournament"];
};

/**
 * Criação e edição de torneio. Na edição de um torneio que já começou, o
 * formato fica travado e não vai no corpo.
 */
export function TournamentForm({ tenant, tournament }: Props) {
  const t = useTranslations("TournamentForm");
  const locale = useLocale();
  const errorMessage = useApiErrorMessage();
  const router = useRouter();
  const [values, setValues] = useState(() =>
    tournament ? valuesFrom(tournament) : defaultValues(locale),
  );
  const onChange = (patch: Partial<TournamentValues>) =>
    setValues((current) => ({ ...current, ...patch }));
  const setupLocked = tournament ? !setupEditable(tournament.status) : false;

  const [error, submit, pending] = useActionState(async () => {
    const texts = {
      name: values.name,
      description: values.description,
      rules: values.rules,
    };
    const result = tournament
      ? await callApi(
          api.PATCH("/tenants/{slug}/tournaments/{tournament}", {
            params: { path: { slug: tenant, tournament: tournament.slug } },
            body: { ...texts, ...(!setupLocked && setupBody(values)) },
          }),
        )
      : await callApi(
          api.POST("/tenants/{slug}/tournaments", {
            params: { path: { slug: tenant } },
            body: {
              ...texts,
              ...setupBody(values),
              slug: values.slug,
              locale: values.locale,
              mode: "duel",
            },
          }),
        );
    if (!result.ok) return errorMessage(result.code, result.params);
    router.push(`/${tenant}/${result.data.slug}`);
    router.refresh();
    return undefined;
  }, undefined);

  return (
    <form action={submit} className="flex flex-col gap-6">
      <TournamentTextFields
        values={values}
        onChange={onChange}
        isNew={!tournament}
      />
      <TournamentSetupFields
        values={values}
        onChange={onChange}
        disabled={setupLocked}
      />
      {setupLocked && <p className="text-sm opacity-70">{t("setupLocked")}</p>}
      {error && <Alert>{error}</Alert>}
      <div>
        <Button type="submit" disabled={pending}>
          {tournament ? t("save") : t("create")}
        </Button>
      </div>
    </form>
  );
}
