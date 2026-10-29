"use client";

import { Form, Stack, Tab, TabPanel, TabPanels, Tabs } from "@carbon/react";
import { useTranslations } from "next-intl";
import { useState } from "react";

import type { components } from "@/shared/api/schema";
import { FormActions } from "@/shared/ui/form-actions";
import { FormError } from "@/shared/ui/form-error";
import { FormSection } from "@/shared/ui/form-section";
import { Notice } from "@/shared/ui/notice";
import { TabList } from "@/shared/ui/tab-list";
import { useTournamentForm } from "../hooks/use-tournament-form";
import { groupOf } from "../lib/form-groups";
import { TournamentChampionFields } from "./tournament-champion-fields";
import { TournamentDescriptionField } from "./tournament-description-field";
import { TournamentFormatFields } from "./tournament-format-fields";
import { TournamentIdentityFields } from "./tournament-identity-fields";
import { TournamentModeFields } from "./tournament-mode-fields";
import { TournamentRegistrationFields } from "./tournament-registration-fields";
import { TournamentRulesSection } from "./tournament-rules-section";
import { TournamentStartField } from "./tournament-start-field";
import { TournamentTranslations } from "./tournament-translations";
import { TournamentVetoFields } from "./tournament-veto-fields";

type Schemas = components["schemas"];

const TABS = ["general", "format", "registration"] as const;
type EditTab = (typeof TABS)[number];

type Props = {
  tenant: string;
  tournament: Schemas["Tournament"];
  /** Os mapas ativos, para o pool do veto. */
  maps: Schemas["Map"][];
};

/**
 * Editar torneio, uma aba por assunto: Geral (nome, descrição, regras
 * adicionais e outros idiomas), Formato (modalidade, chave, séries, campeões,
 * veto) e Inscrições e prazos (início, vagas, visibilidade, check-in,
 * contestação).
 * Um "Salvar" para todas as abas; o campo recusado abre a aba dele. Depois
 * de começar, o motivo dos campos travados vem no topo da aba deles.
 */
export function EditTournamentForm({ tenant, tournament, maps }: Props) {
  const t = useTranslations("TournamentForm");
  const [tab, setTab] = useState<EditTab>("general");
  const form = useTournamentForm(tenant, tournament, {
    reveal: (control) => {
      const group = TABS.find((x) => x === groupOf(control));
      if (group) setTab(group);
    },
  });
  const { values, onChange, errors, setupLocked } = form;

  return (
    <Form
      ref={form.formRef}
      // A conferência é do `check`: ele abre a aba do campo recusado antes de
      // mostrar a mensagem, o que a validação nativa não faz.
      noValidate
      onSubmit={(event) => {
        event.preventDefault();
        if (!form.pending && form.check()) form.submit("save");
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
                <FormSection legend={t("tournament")}>
                  <TournamentIdentityFields
                    tenant={tenant}
                    values={values}
                    onChange={onChange}
                    errors={errors}
                    isNew={false}
                  />
                  <TournamentDescriptionField
                    values={values}
                    onChange={onChange}
                    errors={errors}
                  />
                </FormSection>
                <TournamentRulesSection
                  values={values}
                  onChange={onChange}
                  errors={errors}
                />
                <TournamentTranslations
                  form={form}
                  saved={tournament.translations.map((x) => x.locale)}
                />
              </Stack>
            </TabPanel>
            <TabPanel>
              <Stack gap={7} data-group="format">
                {setupLocked && (
                  <Notice kind="info">{t("formatLocked")}</Notice>
                )}
                <FormSection legend={t("format")}>
                  <TournamentModeFields
                    values={values}
                    onChange={onChange}
                    errors={errors}
                    disabled={setupLocked}
                    saved={tournament}
                  />
                  <TournamentFormatFields
                    values={values}
                    onChange={onChange}
                    errors={errors}
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
                <FormSection legend={t("veto.title")}>
                  <TournamentVetoFields
                    values={values}
                    onChange={onChange}
                    errors={errors}
                    maps={maps}
                    disabled={setupLocked}
                  />
                </FormSection>
              </Stack>
            </TabPanel>
            <TabPanel>
              <Stack gap={6} data-group="registration">
                {setupLocked && (
                  <Notice kind="info">{t("registrationLocked")}</Notice>
                )}
                <TournamentStartField
                  value={values.startsAt}
                  onChange={(startsAt) => onChange({ startsAt })}
                  error={errors.startsAt}
                  checkInMinutes={values.checkInMinutes}
                  disabled={setupLocked}
                />
                <TournamentRegistrationFields
                  values={values}
                  onChange={onChange}
                  errors={errors}
                  setupLocked={setupLocked}
                  finished={false}
                />
              </Stack>
            </TabPanel>
          </TabPanels>
        </Tabs>
        {form.formError && <FormError>{form.formError}</FormError>}
        <FormActions
          pending={form.pending}
          cancelHref={`/${tenant}/${tournament.slug}`}
        />
      </Stack>
    </Form>
  );
}
