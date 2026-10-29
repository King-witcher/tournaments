"use client";

import { Accordion, Form, Stack } from "@carbon/react";
import { useTranslations } from "next-intl";
import { useRef, useState } from "react";

import type { components } from "@/shared/api/schema";
import { Button } from "@/shared/ui/button";
import { ConfirmModal } from "@/shared/ui/confirm-modal";
import { FormActions } from "@/shared/ui/form-actions";
import { FormError } from "@/shared/ui/form-error";
import { FormSection } from "@/shared/ui/form-section";
import {
  type SubmitIntent,
  useTournamentForm,
} from "../hooks/use-tournament-form";
import { useValuesSummary } from "../hooks/use-values-summary";
import { containersOf } from "../lib/form-groups";
import { FormAccordionItem } from "./form-accordion-item";
import { TournamentChampionFields } from "./tournament-champion-fields";
import { TournamentFormatFields } from "./tournament-format-fields";
import { TournamentIdentityFields } from "./tournament-identity-fields";
import { TournamentRegistrationFields } from "./tournament-registration-fields";
import { TournamentReview } from "./tournament-review";
import { TournamentStartField } from "./tournament-start-field";
import { TournamentTextsTabs } from "./tournament-texts-tabs";

type Locale = components["schemas"]["Locale"];

/** Os grupos que começam fechados: têm padrão bom e o resumo no título. */
type Group = "champions" | "registration";

/**
 * Criar torneio, numa página só: à vista o que se decide sempre (nome,
 * endereço, início, chave e séries); fechados, com o resumo no título, os
 * grupos que já vêm com padrão (campeões, inscrições e prazos); por último,
 * descrição e regras, uma aba por idioma. "Criar" confere e mostra a revisão
 * antes de criar já publicado; "Criar como rascunho" cria sem publicar (e
 * sem exigir a data).
 */
export function NewTournamentForm({ tenant }: { tenant: string }) {
  const t = useTranslations("TournamentForm");
  const summary = useValuesSummary();
  const [open, setOpen] = useState<Record<Group, boolean>>({
    champions: false,
    registration: false,
  });
  const [language, setLanguage] = useState<Locale>();
  const [reviewing, setReviewing] = useState(false);
  const [intent, setIntent] = useState<SubmitIntent>();
  const createRef = useRef<HTMLButtonElement>(null);
  const form = useTournamentForm(tenant, undefined, {
    reveal: (control) => {
      const where = containersOf(control);
      if (where.group === "champions" || where.group === "registration") {
        const group: Group = where.group;
        setOpen((o) => ({ ...o, [group]: true }));
      }
      if (where.language) setLanguage(where.language as Locale);
    },
  });
  const { values, onChange, errors } = form;
  const toggle = (group: Group) => (isOpen: boolean) =>
    setOpen((o) => ({ ...o, [group]: isOpen }));
  const send = (next: SubmitIntent) => {
    setIntent(next);
    form.submit(next);
  };

  return (
    <Form
      ref={form.formRef}
      // A conferência é do `check`: ele abre o grupo do campo recusado antes
      // de mostrar a mensagem, o que a validação nativa não faz.
      noValidate
      onSubmit={(event) => {
        event.preventDefault();
        if (!form.pending && form.check()) setReviewing(true);
      }}
    >
      <Stack gap={7}>
        <FormSection legend={t("tournament")}>
          <TournamentIdentityFields
            tenant={tenant}
            values={values}
            onChange={onChange}
            errors={errors}
            isNew
          />
          <TournamentStartField
            value={values.startsAt}
            onChange={(startsAt) => onChange({ startsAt })}
            error={errors.startsAt}
            checkInMinutes={values.checkInMinutes}
            disabled={false}
          />
        </FormSection>
        <FormSection legend={t("format")}>
          <TournamentFormatFields
            values={values}
            onChange={onChange}
            errors={errors}
            disabled={false}
          />
        </FormSection>
        <Accordion align="start">
          <FormAccordionItem
            title={t("champions")}
            summary={summary.champions(values)}
            group="champions"
            open={open.champions}
            onToggle={toggle("champions")}
          >
            <TournamentChampionFields
              values={values}
              onChange={onChange}
              errors={errors}
              disabled={false}
            />
          </FormAccordionItem>
          <FormAccordionItem
            title={t("registration")}
            summary={summary.registration(values)}
            group="registration"
            open={open.registration}
            onToggle={toggle("registration")}
          >
            <TournamentRegistrationFields
              values={values}
              onChange={onChange}
              errors={errors}
              setupLocked={false}
              finished={false}
            />
          </FormAccordionItem>
        </Accordion>
        <FormSection legend={t("texts")} description={t("textsHint")}>
          <TournamentTextsTabs
            form={form}
            language={language ?? form.original}
            onLanguageChange={setLanguage}
          />
        </FormSection>
        {form.formError && <FormError>{form.formError}</FormError>}
        <FormActions
          submitLabel={t("create")}
          submitRef={createRef}
          pending={form.pending && intent === "publish"}
          cancelHref={`/${tenant}`}
          alternative={
            <Button
              kind="secondary"
              pending={form.pending && intent === "draft"}
              onClick={() => {
                if (!form.pending && form.check()) send("draft");
              }}
            >
              {t("createDraft")}
            </Button>
          }
        />
      </Stack>
      <ConfirmModal
        open={reviewing}
        onClose={() => {
          setReviewing(false);
          createRef.current?.focus();
        }}
        onConfirm={() => {
          setReviewing(false);
          createRef.current?.focus();
          send("publish");
        }}
        title={t("review.title")}
        confirmLabel={t("review.confirm")}
        destructive={false}
        details={<TournamentReview form={form} />}
      >
        {t("review.question")}
      </ConfirmModal>
    </Form>
  );
}
