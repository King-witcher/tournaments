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
import { groupOf } from "../lib/form-groups";
import { FormAccordionItem } from "./form-accordion-item";
import { TournamentChampionFields } from "./tournament-champion-fields";
import { TournamentDescriptionField } from "./tournament-description-field";
import { TournamentFormatFields } from "./tournament-format-fields";
import { TournamentIdentityFields } from "./tournament-identity-fields";
import { TournamentRegistrationFields } from "./tournament-registration-fields";
import { TournamentReview } from "./tournament-review";
import { TournamentRulesSection } from "./tournament-rules-section";
import { TournamentStartField } from "./tournament-start-field";
import { TournamentTranslations } from "./tournament-translations";
import { TournamentVetoFields } from "./tournament-veto-fields";

/** Os grupos que começam fechados: têm padrão bom e o resumo no título. */
type Group = "champions" | "veto" | "registration";

/**
 * Criar torneio, numa página só: à vista o que se decide sempre (nome,
 * endereço, início, descrição, chave e séries); fechados, com o resumo no
 * título, os grupos que já vêm com padrão (campeões, veto, inscrições e
 * prazos); depois as regras adicionais e, por último, "Outros idiomas".
 * "Criar" confere e mostra a revisão antes de criar já publicado; "Criar como
 * rascunho" cria sem publicar (e sem exigir a data).
 */
export function NewTournamentForm({
  tenant,
  maps,
}: {
  tenant: string;
  /** Os mapas ativos, para o pool do veto. */
  maps: components["schemas"]["Map"][];
}) {
  const t = useTranslations("TournamentForm");
  const summary = useValuesSummary();
  const [open, setOpen] = useState<Record<Group, boolean>>({
    champions: false,
    veto: false,
    registration: false,
  });
  const [reviewing, setReviewing] = useState(false);
  const [intent, setIntent] = useState<SubmitIntent>();
  const createRef = useRef<HTMLButtonElement>(null);
  const form = useTournamentForm(tenant, undefined, {
    reveal: (control) => {
      const group = groupOf(control);
      if (
        group === "champions" ||
        group === "veto" ||
        group === "registration"
      ) {
        setOpen((o) => ({ ...o, [group]: true }));
      }
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
          <TournamentDescriptionField
            values={values}
            onChange={onChange}
            errors={errors}
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
            title={t("veto.title")}
            summary={summary.veto(values, maps)}
            group="veto"
            open={open.veto}
            onToggle={toggle("veto")}
          >
            <TournamentVetoFields
              values={values}
              onChange={onChange}
              errors={errors}
              maps={maps}
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
        <TournamentRulesSection
          values={values}
          onChange={onChange}
          errors={errors}
        />
        <TournamentTranslations form={form} />
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
        details={<TournamentReview form={form} maps={maps} />}
      >
        {t("review.question")}
      </ConfirmModal>
    </Form>
  );
}
