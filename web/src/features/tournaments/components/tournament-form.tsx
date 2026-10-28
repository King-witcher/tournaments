"use client";

import { useLocale, useTranslations } from "next-intl";
import { useActionState, useState } from "react";

import { useRouter } from "@/i18n/navigation";
import { callApi } from "@/shared/api/call";
import { api } from "@/shared/api/client";
import type { components } from "@/shared/api/schema";
import { useApiErrorMessage } from "@/shared/api/use-api-error-message";
import { submitAction } from "@/shared/lib/submit-action";
import { Alert } from "@/shared/ui/alert";
import { Button } from "@/shared/ui/button";
import {
  createBody,
  defaultValues,
  type TournamentValues,
  updateBody,
  valuesFrom,
} from "../lib/form-values";
import { setupEditable, textsEditable } from "../lib/status";
import { TournamentContestField } from "./tournament-contest-field";
import { TournamentScheduleFields } from "./tournament-schedule-fields";
import { TournamentSetupFields } from "./tournament-setup-fields";
import { TournamentTextFields } from "./tournament-text-fields";
import { TournamentVisibilityField } from "./tournament-visibility-field";

type Props = {
  tenant: string;
  /** Sem torneio, cria um rascunho; com torneio, edita. */
  tournament?: components["schemas"]["Tournament"];
};

/**
 * Criação e edição de torneio. Na edição de um torneio que já começou, o
 * formato, a data e o check-in ficam travados e não vão no corpo.
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
  const finished = tournament ? !textsEditable(tournament.status) : false;
  const published = tournament ? tournament.status !== "draft" : false;

  const [error, submit, pending] = useActionState(async () => {
    const result = tournament
      ? await callApi(
          api.PATCH("/tenants/{slug}/tournaments/{tournament}", {
            params: { path: { slug: tenant, tournament: tournament.slug } },
            body: updateBody(values, setupLocked),
          }),
        )
      : await callApi(
          api.POST("/tenants/{slug}/tournaments", {
            params: { path: { slug: tenant } },
            body: createBody(values),
          }),
        );
    if (!result.ok) return errorMessage(result.code, result.params);
    router.push(`/${tenant}/${result.data.slug}`);
    router.refresh();
    return undefined;
  }, undefined);

  return (
    <form onSubmit={submitAction(submit)} className="flex flex-col gap-6">
      <TournamentTextFields
        values={values}
        onChange={onChange}
        isNew={!tournament}
      />
      <TournamentVisibilityField
        value={values.visibility}
        onChange={(visibility) => onChange({ visibility })}
        disabled={finished}
      />
      <TournamentScheduleFields
        values={values}
        onChange={onChange}
        disabled={setupLocked}
        required={published}
      />
      <TournamentSetupFields
        values={values}
        onChange={onChange}
        disabled={setupLocked}
      />
      <TournamentContestField
        value={values.contestMinutes}
        onChange={(contestMinutes) => onChange({ contestMinutes })}
        disabled={finished}
      />
      {setupLocked && <p className="text-sm text-muted">{t("setupLocked")}</p>}
      {error && <Alert>{error}</Alert>}
      <div>
        <Button type="submit" pending={pending}>
          {tournament ? t("save") : t("create")}
        </Button>
      </div>
    </form>
  );
}
