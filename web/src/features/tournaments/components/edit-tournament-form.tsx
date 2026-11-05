"use client";

import { useTranslations } from "next-intl";
import { type ReactNode, useRef, useState } from "react";

import type { components } from "@/shared/api/schema";
import { Form } from "@/shared/ui/carbon/form";
import { Stack } from "@/shared/ui/carbon/stack";
import { Tab, TabPanel, TabPanels, Tabs } from "@/shared/ui/carbon/tabs";
import { ConfirmModal } from "@/shared/ui/confirm-modal";
import { FormActions } from "@/shared/ui/form-actions";
import { FormError } from "@/shared/ui/form-error";
import { FormSection } from "@/shared/ui/form-section";
import { Notice } from "@/shared/ui/notice";
import { TabList } from "@/shared/ui/tab-list";
import { useTournamentForm } from "../hooks/use-tournament-form";
import { groupOf } from "../lib/form-groups";
import { isTdm } from "../lib/form-values";
import { isFinished } from "../lib/status";
import { TournamentChampionFields } from "./tournament-champion-fields";
import { TournamentDescriptionField } from "./tournament-description-field";
import { TournamentFormatFields } from "./tournament-format-fields";
import { TournamentIdentityFields } from "./tournament-identity-fields";
import { TournamentModeFields } from "./tournament-mode-fields";
import { TournamentRegistrationFields } from "./tournament-registration-fields";
import { TournamentResultsFields } from "./tournament-results-fields";
import { TournamentRulesSection } from "./tournament-rules-section";
import { TournamentSlotsField } from "./tournament-slots-field";
import { TournamentStartField } from "./tournament-start-field";
import { TournamentTeamRulesFields } from "./tournament-team-rules-fields";
import { TournamentTranslations } from "./tournament-translations";
import { TournamentVetoFields } from "./tournament-veto-fields";

type Schemas = components["schemas"];

const TABS = ["general", "format", "registration", "results"] as const;
type EditTab = (typeof TABS)[number];

type Props = {
  tenant: string;
  tournament: Schemas["Tournament"];
  /** Os mapas ativos, para o pool do veto. */
  maps: Schemas["Map"][];
  /** A capa (`CoverSection`), que se salva sozinha, na aba Geral. */
  cover?: ReactNode;
  /** Se esta API tem capturas de tela. */
  screenshots: boolean;
};

/**
 * Editar torneio, uma aba por assunto: Geral (nome, descrição, capa, regras
 * adicionais e outros idiomas), Formato (modalidade, vagas, chave, séries,
 * quem age pelo time, mapas, campeões), Inscrições e prazos (início, inscrição, check-in) e
 * Resultados (contestação, captura de tela).
 * Um "Salvar" para todas as abas; o campo recusado abre a aba dele. Depois
 * de começar, o motivo dos campos travados vem no topo da aba deles; encerrado
 * o torneio, o mesmo motivo em todas, com o nome, a descrição e a capa ainda
 * livres. O endereço só muda no rascunho, e salvar com ele trocado pede a
 * confirmação do que isso quebra.
 */
export function EditTournamentForm({
  tenant,
  tournament,
  maps,
  cover,
  screenshots,
}: Props) {
  const t = useTranslations("TournamentForm");
  const [tab, setTab] = useState<EditTab>("general");
  const [asking, setAsking] = useState(false);
  const submitButton = useRef<HTMLButtonElement>(null);
  const form = useTournamentForm(tenant, tournament, {
    reveal: (control) => {
      const group = TABS.find((x) => x === groupOf(control));
      if (group) setTab(group);
    },
  });
  const { values, onChange, errors, setupLocked } = form;
  const finished = isFinished(tournament.status);
  const locked = finished ? t("finishedLocked") : undefined;

  return (
    <Form
      ref={form.formRef}
      // A conferência é do `check`: ele abre a aba do campo recusado antes de
      // mostrar a mensagem, o que a validação nativa não faz.
      noValidate
      onSubmit={(event) => {
        event.preventDefault();
        if (form.pending || !form.check()) return;
        if (form.movesAddress) setAsking(true);
        else form.submit("save");
      }}
    >
      <Stack gap={7}>
        <Tabs
          selectedIndex={TABS.indexOf(tab)}
          onChange={({ selectedIndex }) => setTab(TABS[selectedIndex])}
        >
          <TabList aria-label={t("tabs.label")}>
            {TABS.map((key) => (
              <Tab key={key}>{t(`tabs.${key}`)}</Tab>
            ))}
          </TabList>
          <TabPanels>
            <TabPanel>
              <Stack gap={7} data-group="general">
                {locked && <Notice kind="info">{locked}</Notice>}
                <FormSection legend={t("tournament")}>
                  <TournamentIdentityFields
                    tenant={tenant}
                    values={values}
                    onChange={onChange}
                    errors={errors}
                    isNew={false}
                    slugLocked={tournament.status !== "draft"}
                  />
                  <TournamentDescriptionField
                    values={values}
                    onChange={onChange}
                    errors={errors}
                  />
                </FormSection>
                {cover}
                <TournamentRulesSection
                  values={values}
                  onChange={onChange}
                  errors={errors}
                  disabled={finished}
                />
                <TournamentTranslations
                  form={form}
                  saved={tournament.translations.map((x) => x.locale)}
                  rulesLocked={finished}
                />
              </Stack>
            </TabPanel>
            <TabPanel>
              <Stack gap={7} data-group="format">
                {setupLocked && (
                  <Notice kind="info">{locked ?? t("formatLocked")}</Notice>
                )}
                <FormSection legend={t("format")}>
                  <TournamentModeFields
                    values={values}
                    onChange={onChange}
                    errors={errors}
                    disabled={setupLocked}
                    saved={tournament}
                  />
                  <TournamentSlotsField
                    values={values}
                    onChange={onChange}
                    errors={errors}
                    disabled={setupLocked}
                  />
                  <TournamentFormatFields
                    values={values}
                    onChange={onChange}
                    errors={errors}
                    disabled={setupLocked}
                  />
                </FormSection>
                {isTdm(values) && (
                  <FormSection legend={t("teamRules.title")}>
                    <TournamentTeamRulesFields
                      values={values}
                      onChange={onChange}
                      disabled={setupLocked}
                    />
                  </FormSection>
                )}
                <FormSection legend={t("veto.title")}>
                  <TournamentVetoFields
                    tenant={tenant}
                    values={values}
                    onChange={onChange}
                    errors={errors}
                    maps={maps}
                    disabled={setupLocked}
                  />
                </FormSection>
                <FormSection legend={t("champions")}>
                  <TournamentChampionFields
                    values={values}
                    onChange={onChange}
                    errors={errors}
                    disabled={setupLocked}
                  />
                </FormSection>
              </Stack>
            </TabPanel>
            <TabPanel>
              <Stack gap={6} data-group="registration">
                {setupLocked && (
                  <Notice kind="info">
                    {locked ?? t("registrationLocked")}
                  </Notice>
                )}
                <TournamentStartField
                  values={values}
                  onChange={onChange}
                  errors={errors}
                  disabled={setupLocked}
                  saved={tournament.starts_at ?? null}
                />
                <TournamentRegistrationFields
                  values={values}
                  onChange={onChange}
                  errors={errors}
                  setupLocked={setupLocked}
                  finished={finished}
                />
              </Stack>
            </TabPanel>
            <TabPanel>
              <Stack gap={6} data-group="results">
                {locked && <Notice kind="info">{locked}</Notice>}
                <TournamentResultsFields
                  values={values}
                  onChange={onChange}
                  errors={errors}
                  finished={finished}
                  screenshots={screenshots}
                />
              </Stack>
            </TabPanel>
          </TabPanels>
        </Tabs>
        {form.formError && <FormError>{form.formError}</FormError>}
        <FormActions
          submitRef={submitButton}
          pending={form.pending}
          disabled={!form.canSubmit}
          cancelHref={`/${tenant}/${form.savedSlug}`}
        />
      </Stack>
      <ConfirmModal
        open={asking}
        title={t("slugConfirm.title")}
        confirmLabel={t("slugConfirm.submit")}
        destructive={false}
        onConfirm={() => {
          setAsking(false);
          form.submit("save");
        }}
        onClose={() => {
          setAsking(false);
          // O Modal sai da árvore e o foco cairia no body.
          requestAnimationFrame(() => submitButton.current?.focus());
        }}
      >
        {t("slugConfirm.text", {
          old: form.savedSlug ?? tournament.slug,
          address: values.slug,
        })}
      </ConfirmModal>
    </Form>
  );
}
