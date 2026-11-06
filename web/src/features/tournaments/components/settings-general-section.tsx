"use client";

import { useTranslations } from "next-intl";
import { type ReactNode, useRef, useState } from "react";

import { ConfirmModal } from "@/shared/ui/confirm-modal";
import { useSettingsSection } from "../hooks/use-settings-section";
import { useTournamentSettings } from "../hooks/use-tournament-settings";
import { SettingsSection } from "./settings-section";
import { TournamentDescriptionField } from "./tournament-description-field";
import { TournamentIdentityFields } from "./tournament-identity-fields";

type Props = {
  /** A capa (`CoverSection`), que se salva sozinha, depois do Salvar. */
  cover: ReactNode;
};

/**
 * Geral: nome, endereço e descrição, com um Salvar, e a capa embaixo, que se
 * salva sozinha; no título, o nome e o endereço salvos. O endereço só muda no
 * rascunho, e salvar com ele trocado pede a confirmação do que isso quebra;
 * depois de publicar, ele fica só para leitura, com o motivo antes dos campos.
 */
export function SettingsGeneralSection({ cover }: Props) {
  const t = useTranslations("TournamentForm");
  const tSettings = useTranslations("TournamentSettings");
  const { tenant, tournament } = useTournamentSettings();
  const form = useSettingsSection("general");
  const { values, onChange, errors } = form;
  const [asking, setAsking] = useState(false);
  const submitRef = useRef<HTMLButtonElement>(null);
  const draft = tournament.status === "draft";
  const moves = draft && values.slug !== tournament.slug;

  return (
    <>
      <SettingsSection
        section="general"
        form={form}
        summary={`${form.saved.name} · ${form.saved.slug}`}
        lock={draft ? undefined : tSettings("locks.slug")}
        onSave={() => (moves ? setAsking(true) : form.submit())}
        submitRef={submitRef}
        after={cover}
      >
        <TournamentIdentityFields
          tenant={tenant}
          values={values}
          onChange={onChange}
          errors={errors}
          isNew={false}
          slugReadOnly={!draft}
        />
        <TournamentDescriptionField
          values={values}
          onChange={onChange}
          errors={errors}
        />
      </SettingsSection>
      <ConfirmModal
        open={asking}
        title={t("slugConfirm.title")}
        confirmLabel={t("slugConfirm.submit")}
        destructive={false}
        onConfirm={() => {
          setAsking(false);
          requestAnimationFrame(() => submitRef.current?.focus());
          form.submit();
        }}
        onClose={() => {
          setAsking(false);
          // O Modal sai da árvore e o foco cairia no body.
          requestAnimationFrame(() => submitRef.current?.focus());
        }}
      >
        {t("slugConfirm.text", { old: tournament.slug, address: values.slug })}
      </ConfirmModal>
    </>
  );
}
