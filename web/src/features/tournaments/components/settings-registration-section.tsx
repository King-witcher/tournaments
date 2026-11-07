"use client";

import { useTranslations } from "next-intl";

import { FormRow } from "@/shared/ui/form-row";
import { LocalDateTime } from "@/shared/ui/local-date-time";
import { useSettingsSection } from "../hooks/use-settings-section";
import { useTournamentSettings } from "../hooks/use-tournament-settings";
import { useValuesSummary } from "../hooks/use-values-summary";
import { isFinished, setupEditable } from "../lib/status";
import { SettingsSection } from "./settings-section";
import { TournamentRegistrationFields } from "./tournament-registration-fields";
import { TournamentStartField } from "./tournament-start-field";

/**
 * Início e inscrições: o início (o único sem padrão, por isso logo depois de
 * Geral) numa linha e, na de baixo, o tipo de inscrição e o check-in lado a
 * lado. Depois de começar, só a inscrição muda (o motivo diz o que não muda);
 * encerrado, nada.
 */
export function SettingsRegistrationSection() {
  const tSettings = useTranslations("TournamentSettings");
  const tReview = useTranslations("TournamentForm.review");
  const summary = useValuesSummary();
  const { tournament } = useTournamentSettings();
  const form = useSettingsSection("registration");
  const { values, onChange, errors, saved } = form;
  const finished = isFinished(tournament.status);
  const scheduleLocked = !setupEditable(tournament.status);

  return (
    <SettingsSection
      section="registration"
      form={form}
      summary={
        <>
          {saved.startsAt ? (
            <LocalDateTime value={saved.startsAt} />
          ) : (
            tReview("noDate")
          )}
          {` · ${summary.registration(saved)}`}
        </>
      }
      lock={
        finished
          ? tSettings("locks.finished")
          : scheduleLocked
            ? tSettings("locks.schedule")
            : undefined
      }
      readOnly={finished}
      onSave={form.submit}
    >
      <TournamentStartField
        values={values}
        onChange={onChange}
        errors={errors}
        readOnly={scheduleLocked}
        saved={tournament.starts_at ?? null}
      />
      <FormRow columns={2}>
        <TournamentRegistrationFields
          values={values}
          onChange={onChange}
          errors={errors}
          checkInReadOnly={scheduleLocked}
          admissionReadOnly={finished}
        />
      </FormRow>
    </SettingsSection>
  );
}
