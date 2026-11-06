"use client";

import { useTranslations } from "next-intl";

import { useSettingsSection } from "../hooks/use-settings-section";
import { useTournamentSettings } from "../hooks/use-tournament-settings";
import { isFinished, setupEditable } from "../lib/status";
import { SettingsSection } from "./settings-section";
import { TournamentRegistrationFields } from "./tournament-registration-fields";
import { TournamentStartField } from "./tournament-start-field";

/**
 * Inscrições e prazos: o início, o tipo de inscrição e o check-in. Depois de
 * começar, só a inscrição muda (o motivo diz o que não muda); encerrado, nada.
 */
export function SettingsRegistrationSection() {
  const tSettings = useTranslations("TournamentSettings");
  const { tournament } = useTournamentSettings();
  const form = useSettingsSection("registration");
  const { values, onChange, errors } = form;
  const finished = isFinished(tournament.status);
  const scheduleLocked = !setupEditable(tournament.status);

  return (
    <SettingsSection
      section="registration"
      form={form}
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
      <TournamentRegistrationFields
        values={values}
        onChange={onChange}
        errors={errors}
        checkInReadOnly={scheduleLocked}
        admissionReadOnly={finished}
      />
    </SettingsSection>
  );
}
