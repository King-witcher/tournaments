"use client";

import { useTranslations } from "next-intl";
import type { ReactNode } from "react";

import { LocalDateTime } from "@/shared/ui/local-date-time";
import { useSettingsSection } from "../hooks/use-settings-section";
import { useTournamentSettings } from "../hooks/use-tournament-settings";
import { useValuesSummary } from "../hooks/use-values-summary";
import { isFinished, setupEditable } from "../lib/status";
import { SettingsSection } from "./settings-section";
import { TournamentRegistrationFields } from "./tournament-registration-fields";
import { TournamentStartField } from "./tournament-start-field";

/**
 * Início e inscrições, na largura de leitura: o início (o único sem padrão,
 * por isso logo depois de Geral), o tipo de inscrição e o check-in e, ao lado,
 * o link de inscrição (`invite`), que não entra no Salvar. Depois de começar,
 * só a inscrição muda (o motivo diz o que não muda); encerrado, nada.
 */
export function SettingsRegistrationSection({ invite }: { invite: ReactNode }) {
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
      aside={invite}
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
