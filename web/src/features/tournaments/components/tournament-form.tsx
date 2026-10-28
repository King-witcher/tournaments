"use client";

import { Form, Stack } from "@carbon/react";
import { useTranslations } from "next-intl";

import type { components } from "@/shared/api/schema";
import { submitAction } from "@/shared/lib/submit-action";
import { FormActions } from "@/shared/ui/form-actions";
import { FormError } from "@/shared/ui/form-error";
import { Notice } from "@/shared/ui/notice";
import { useTournamentForm } from "../hooks/use-tournament-form";
import { textsEditable } from "../lib/status";
import { TournamentIdentityFields } from "./tournament-identity-fields";
import { TournamentScheduleFields } from "./tournament-schedule-fields";
import { TournamentSetupFields } from "./tournament-setup-fields";
import { TournamentTextFields } from "./tournament-text-fields";
import { TranslationFields } from "./translation-fields";

type Props = {
  tenant: string;
  /** Sem torneio, cria um rascunho; com torneio, edita (e traduz). */
  tournament?: components["schemas"]["Tournament"];
};

/**
 * Criação e edição de torneio, uma seção por grupo de campos. Na edição de
 * um torneio que já começou, formato, vagas, data e check-in ficam travados
 * (e não vão no corpo), com o motivo antes deles.
 */
export function TournamentForm({ tenant, tournament }: Props) {
  const t = useTranslations("TournamentForm");
  const form = useTournamentForm(tenant, tournament);
  const { values, onChange, errors } = form;
  const finished = tournament ? !textsEditable(tournament.status) : false;
  const isNew = tournament === undefined;

  return (
    <Form ref={form.formRef} onSubmit={submitAction(form.submit)}>
      <Stack gap={7}>
        <TournamentIdentityFields
          tenant={tenant}
          values={values}
          onChange={onChange}
          errors={errors}
          isNew={isNew}
          finished={finished}
        />
        {/* O motivo antes dos campos travados, não no fim do formulário: quem
            rola vê os campos desabilitados e já sabe por quê. */}
        {form.setupLocked && <Notice kind="info">{t("setupLocked")}</Notice>}
        <TournamentSetupFields
          values={values}
          onChange={onChange}
          errors={errors}
          disabled={form.setupLocked}
        />
        <TournamentScheduleFields
          values={values}
          onChange={onChange}
          errors={errors}
          setupLocked={form.setupLocked}
          published={tournament !== undefined && tournament.status !== "draft"}
          finished={finished}
        />
        <TournamentTextFields
          values={values}
          onChange={onChange}
          errors={errors}
          isNew={isNew}
        />
        {tournament &&
          form.others.map((locale) => (
            <TranslationFields
              key={locale}
              tenant={tenant}
              tournament={tournament}
              locale={locale}
              values={form.translationOf(locale)}
              onChange={(patch) => form.onTranslationChange(locale, patch)}
              errors={form.translationErrors(locale)}
              onRemoved={() => form.clearTranslation(locale)}
            />
          ))}
        {form.formError && <FormError>{form.formError}</FormError>}
        <FormActions
          submitLabel={isNew ? t("create") : undefined}
          pending={form.pending}
          cancelHref={isNew ? `/${tenant}` : `/${tenant}/${tournament.slug}`}
        />
      </Stack>
    </Form>
  );
}
