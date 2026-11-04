"use client";

import { Accordion, Form, Stack } from "@carbon/react";
import { useTranslations } from "next-intl";
import {
  useCallback,
  useEffect,
  useEffectEvent,
  useRef,
  useState,
} from "react";
import { ImageNotSent } from "@/features/images/components/image-not-sent";
import { ImagePicker } from "@/features/images/components/image-picker";
import { useImageOnCreate } from "@/features/images/hooks/use-image-on-create";
import { useImageSettings } from "@/features/images/hooks/use-image-settings";
import { useScreenshotSettings } from "@/features/screenshots/hooks/use-screenshot-settings";
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
import { isTdm } from "../lib/form-values";
import { rememberedPool } from "../lib/remembered-pool";
import { FormAccordionItem } from "./form-accordion-item";
import { TournamentChampionFields } from "./tournament-champion-fields";
import { TournamentDescriptionField } from "./tournament-description-field";
import { TournamentFormatFields } from "./tournament-format-fields";
import { TournamentIdentityFields } from "./tournament-identity-fields";
import { TournamentModeFields } from "./tournament-mode-fields";
import { TournamentRegistrationFields } from "./tournament-registration-fields";
import { TournamentResultsFields } from "./tournament-results-fields";
import { TournamentReview } from "./tournament-review";
import { TournamentRulesSection } from "./tournament-rules-section";
import { TournamentSlotsField } from "./tournament-slots-field";
import { TournamentSourceField } from "./tournament-source-field";
import { TournamentStartField } from "./tournament-start-field";
import { TournamentTeamRulesFields } from "./tournament-team-rules-fields";
import { TournamentTranslations } from "./tournament-translations";
import { TournamentVetoFields } from "./tournament-veto-fields";

/** Os grupos que começam fechados: têm padrão bom ou são opcionais, e o resumo no título. */
const GROUPS = [
  "teams",
  "veto",
  "champions",
  "registration",
  "results",
  "rules",
  "languages",
] as const;
type Group = (typeof GROUPS)[number];

type Schemas = components["schemas"];

const CLOSED = Object.fromEntries(GROUPS.map((g) => [g, false])) as Record<
  Group,
  boolean
>;

/**
 * Criar torneio, numa página só: à vista o que se decide sempre (nome,
 * endereço, início, descrição, modalidade, vagas, chave e séries); fechados,
 * com o resumo no título, os grupos que já vêm com padrão (nos times, quem age
 * por eles; mapas, campeões, inscrições e prazos, resultados) e os opcionais
 * (regras adicionais e outros idiomas). Os botões só liberam com todos os campos válidos. "Criar como
 * rascunho" é o primário (e o Enter): cria sem publicar. "Criar e publicar"
 * confere e mostra a revisão antes, porque publicado não volta a rascunho.
 * No topo, "Copiar de" começa da configuração de outro torneio. A capa
 * opcional só vai ao bucket depois de o torneio ser criado
 * (`useImageOnCreate`).
 */
export function NewTournamentForm({
  tenant,
  maps,
  tournaments,
  source,
}: {
  tenant: string;
  /** Os mapas ativos, para o pool do veto. */
  maps: Schemas["Map"][];
  /** Os torneios da comunidade, para copiar um deles. */
  tournaments: Schemas["TournamentSummary"][];
  /** O torneio copiado ao abrir (`?from=`). */
  source: Schemas["Tournament"] | undefined;
}) {
  const t = useTranslations("TournamentForm");
  const tLocales = useTranslations("Locales");
  const summary = useValuesSummary();
  const screenshots = useScreenshotSettings()?.enabled ?? false;
  const tCover = useTranslations("Images.cover");
  const images = useImageSettings();
  const cover = useImageOnCreate();
  // A tradução copiada vem sem o nome, que é obrigatório: o grupo abre para
  // ele ficar à vista.
  const [open, setOpen] = useState(() => ({
    ...CLOSED,
    languages: (source?.translations.length ?? 0) > 0,
  }));
  const [reviewing, setReviewing] = useState(false);
  const [intent, setIntent] = useState<SubmitIntent>();
  const publishRef = useRef<HTMLButtonElement>(null);
  const form = useTournamentForm(tenant, undefined, {
    reveal: (control) => {
      const group = GROUPS.find((g) => g === groupOf(control));
      if (group) setOpen((o) => ({ ...o, [group]: true }));
    },
    source,
    maps,
    afterCreate: (slug) =>
      cover.send({ kind: "cover", tenant, tournament: slug }),
  });
  const { values, onChange, errors, copyFrom } = form;
  // O pool lembrado entra depois da hidratação: o servidor não lê o
  // `localStorage`, e o primeiro desenho tem de ser igual ao dele. Com
  // origem, o pool é o dela.
  const restorePool = useEffectEvent(() => {
    if (source) return;
    const pool = rememberedPool(
      tenant,
      maps.map((m) => m.slug),
    );
    if (pool) onChange({ mapPool: pool });
  });
  useEffect(() => restorePool(), []);
  const copy = useCallback(
    (next: Schemas["Tournament"] | undefined) => {
      copyFrom(next);
      if (next?.translations.length)
        setOpen((o) => ({ ...o, languages: true }));
    },
    [copyFrom],
  );
  const toggle = (group: Group) => (isOpen: boolean) =>
    setOpen((o) => ({ ...o, [group]: isOpen }));
  const send = (next: SubmitIntent) => {
    setIntent(next);
    form.submit(next);
  };
  const languages = form.others
    .filter((l) => form.translationOf(l).enabled)
    .map((l) => tLocales(l));

  if (cover.unsent?.target.kind === "cover") {
    const { tournament } = cover.unsent.target;
    return (
      <ImageNotSent unsent={cover.unsent} href={`/${tenant}/${tournament}`} />
    );
  }

  return (
    <Form
      ref={form.formRef}
      // A conferência é do `check`: ele abre o grupo do campo recusado antes
      // de mostrar a mensagem, o que a validação nativa não faz.
      noValidate
      onSubmit={(event) => {
        event.preventDefault();
        if (!form.pending && form.check()) send("draft");
      }}
    >
      <Stack gap={7}>
        {tournaments.length > 0 && (
          <TournamentSourceField
            tenant={tenant}
            tournaments={tournaments}
            initial={source?.slug}
            onCopy={copy}
          />
        )}
        <FormSection legend={t("tournament")}>
          <TournamentIdentityFields
            tenant={tenant}
            values={values}
            onChange={onChange}
            errors={errors}
            isNew
          />
          <TournamentStartField
            values={values}
            onChange={onChange}
            errors={errors}
            disabled={false}
          />
          <TournamentDescriptionField
            values={values}
            onChange={onChange}
            errors={errors}
          />
          {images?.enabled && (
            <ImagePicker
              kind="cover"
              maxBytes={images.max_bytes}
              value={cover.image}
              onChange={cover.setImage}
              label={tCover("title")}
              hint={tCover("hint")}
            />
          )}
        </FormSection>
        <FormSection legend={t("format")}>
          <TournamentModeFields
            values={values}
            onChange={onChange}
            errors={errors}
            disabled={false}
          />
          <TournamentSlotsField
            values={values}
            onChange={onChange}
            errors={errors}
            disabled={false}
          />
          <TournamentFormatFields
            values={values}
            onChange={onChange}
            errors={errors}
            disabled={false}
          />
        </FormSection>
        <Accordion>
          {isTdm(values) && (
            <FormAccordionItem
              title={t("teamRules.title")}
              summary={summary.teamRules(values)}
              group="teams"
              open={open.teams}
              onToggle={toggle("teams")}
            >
              <TournamentTeamRulesFields
                values={values}
                onChange={onChange}
                disabled={false}
              />
            </FormAccordionItem>
          )}
          {/* Mapas antes de Campeões: a ordem em que a série os decide. */}
          <FormAccordionItem
            title={t("veto.title")}
            summary={summary.veto(values, maps)}
            group="veto"
            open={open.veto}
            onToggle={toggle("veto")}
          >
            <TournamentVetoFields
              tenant={tenant}
              values={values}
              onChange={onChange}
              errors={errors}
              maps={maps}
              disabled={false}
            />
          </FormAccordionItem>
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
          <FormAccordionItem
            title={t("results")}
            summary={summary.results(values, screenshots)}
            group="results"
            open={open.results}
            onToggle={toggle("results")}
          >
            <TournamentResultsFields
              values={values}
              onChange={onChange}
              errors={errors}
              finished={false}
            />
          </FormAccordionItem>
          <FormAccordionItem
            title={t("rules")}
            summary={summary.rules(values)}
            group="rules"
            open={open.rules}
            onToggle={toggle("rules")}
          >
            <TournamentRulesSection
              values={values}
              onChange={onChange}
              errors={errors}
              bare
            />
          </FormAccordionItem>
          <FormAccordionItem
            title={t("otherLanguages")}
            summary={summary.languages(languages)}
            group="languages"
            open={open.languages}
            onToggle={toggle("languages")}
          >
            <TournamentTranslations form={form} bare />
          </FormAccordionItem>
        </Accordion>
        {form.formError && <FormError>{form.formError}</FormError>}
        <FormActions
          submitLabel={t("createDraft")}
          disabled={!form.canSubmit}
          pending={form.pending && intent === "draft"}
          cancelHref={`/${tenant}`}
          alternative={
            <Button
              ref={publishRef}
              kind="secondary"
              disabled={!form.canSubmit}
              pending={form.pending && intent === "publish"}
              onClick={() => {
                if (!form.pending && form.check()) setReviewing(true);
              }}
            >
              {t("create")}
            </Button>
          }
        />
      </Stack>
      <ConfirmModal
        open={reviewing}
        onClose={() => {
          setReviewing(false);
          publishRef.current?.focus();
        }}
        onConfirm={() => {
          setReviewing(false);
          publishRef.current?.focus();
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
