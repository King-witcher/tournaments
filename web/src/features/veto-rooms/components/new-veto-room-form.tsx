"use client";

import { Accordion, Form, Stack } from "@carbon/react";
import { useLocale, useTranslations } from "next-intl";
import { useActionState, useEffect, useState } from "react";

import { FormAccordionItem } from "@/features/tournaments/components/form-accordion-item";
import { MapPoolField } from "@/features/tournaments/components/map-pool-field";
import { SecondsSelect } from "@/features/tournaments/components/seconds-select";
import { TournamentChampionFields } from "@/features/tournaments/components/tournament-champion-fields";
import { TournamentVetoOrderField } from "@/features/tournaments/components/tournament-veto-order-field";
import { useValuesSummary } from "@/features/tournaments/hooks/use-values-summary";
import type { FieldErrors } from "@/features/tournaments/lib/form-errors";
import {
  defaultValues,
  type NoRepeatChoice,
  poolOf,
  STEP_SECONDS,
  stepSecondsGroup,
  type TournamentValues,
  teamSizeOf,
} from "@/features/tournaments/lib/form-values";
import { rememberedPool } from "@/features/tournaments/lib/remembered-pool";
import { useRouter } from "@/i18n/navigation";
import { callApi } from "@/shared/api/call";
import { api } from "@/shared/api/client";
import type { components } from "@/shared/api/schema";
import { useApiErrorMessage } from "@/shared/api/use-api-error-message";
import { ButtonLink } from "@/shared/ui/button-link";
import { FormActions } from "@/shared/ui/form-actions";
import { FormError } from "@/shared/ui/form-error";
import { Notice } from "@/shared/ui/notice";
import { RadioGroup } from "@/shared/ui/radio-group";

const TEAM_SIZES = ["1", "2", "3", "4"];
const BEST_OF = ["1", "3", "5"];
const MAP_MODES = ["veto", "off"] as const;
/** Na sala, a repetição só vale dentro da série. */
const REPEAT_CHOICES: NoRepeatChoice[] = ["off", "series"];
const NO_ERRORS: FieldErrors = {};
/** O pool lembrado desta tela: nenhuma comunidade usa este endereço (`veto` é reservado). */
const POOL_KEY = "veto";

/** O erro que pede a conta: o teto de salas sem login e o limite por IP. */
const OFFERS_LOGIN = ["veto_room.anonymous_full", "veto_room.rate_limited"];

type Props = {
  /** Os mapas ativos (`GET /maps`). */
  maps: components["schemas"]["Map"][];
  signedIn: boolean;
};

type Failure = { message: string; login: boolean };

/**
 * Criar uma sala: o modo e o melhor de N à vista, e o resto (os mesmos campos
 * do formulário de torneio) em dois grupos fechados, com o resumo no título.
 * Criada, a pessoa vai para a página de gestão, com os links.
 */
export function NewVetoRoomForm({ maps, signedIn }: Props) {
  const t = useTranslations("VetoRoom.form");
  const tTournament = useTranslations("TournamentForm");
  const tAuth = useTranslations("Auth");
  const locale = useLocale();
  const router = useRouter();
  const errorMessage = useApiErrorMessage();
  const summary = useValuesSummary();
  const [values, setValues] = useState<TournamentValues>(() => ({
    ...defaultValues(locale as components["schemas"]["Locale"]),
    // Uma série só: o melhor de N vale para ela inteira.
    stageFormat: "round_robin",
    finalsBestOf: 3,
  }));
  const [open, setOpen] = useState({ maps: false, champions: false });
  const onChange = (patch: Partial<TournamentValues>) =>
    setValues((v) => ({ ...v, ...patch }));

  // Só no navegador: o servidor desenha o padrão, e o resumo não pode divergir.
  useEffect(() => {
    const active = maps.map((m) => m.slug);
    const pool = rememberedPool(POOL_KEY, active);
    if (pool) setValues((v) => ({ ...v, mapPool: pool }));
  }, [maps]);

  const pool = poolOf(values, maps);
  const poolTooSmall = values.mapMode === "veto" && pool.length < values.bestOf;
  const noPhases = values.mapMode === "off" && !values.championPhase;

  const [failure, submit, pending] = useActionState<Failure | undefined>(
    async () => {
      const outcome = await callApi(
        api.POST("/veto-rooms", {
          body: {
            team_size: teamSizeOf(values),
            best_of: values.bestOf,
            veto: {
              map_mode: values.mapMode,
              order: values.vetoOrder,
              map_pool: values.mapMode === "veto" ? pool : undefined,
              champion_phase: values.championPhase,
              step_seconds: values.vetoSeconds,
              team_pick_seconds: values.teamPickSeconds,
            },
            champion_rules: {
              bans_per_series: values.bansPerSeries,
              mirror: values.mirror,
              same_team: values.sameTeam,
              no_repeat: values.noRepeat === "off" ? null : values.noRepeatWho,
            },
          },
        }),
      );
      if (outcome.ok) {
        router.push(`/veto/${outcome.data.token}`);
        return undefined;
      }
      return {
        message: errorMessage(outcome.code, outcome.params),
        login: !signedIn && OFFERS_LOGIN.includes(outcome.code),
      };
    },
    undefined,
  );

  return (
    <Form action={submit}>
      <Stack gap={7}>
        <Stack gap={6}>
          <RadioGroup
            legendText={t("mode")}
            name="teamSize"
            orientation="horizontal"
            value={String(teamSizeOf(values))}
            options={TEAM_SIZES.map((size) => ({
              value: size,
              label:
                size === "1" ? t("duel") : t("teams", { size: Number(size) }),
            }))}
            onChange={(size) =>
              onChange({
                mode: size === "1" ? "duel" : "tdm",
                teamSize: Number(size),
              })
            }
          />
          <RadioGroup
            legendText={t("bestOf")}
            name="bestOf"
            orientation="horizontal"
            value={String(values.bestOf)}
            options={BEST_OF.map((n) => ({ value: n, label: n }))}
            onChange={(n) =>
              onChange({ bestOf: Number(n), finalsBestOf: Number(n) })
            }
          />
        </Stack>
        <Accordion>
          {/* Mapas antes de Campeões: a ordem em que a sala os decide. */}
          <FormAccordionItem
            title={tTournament("veto.title")}
            summary={summary.veto(values, maps)}
            group="maps"
            open={open.maps || poolTooSmall}
            onToggle={(next) => setOpen((o) => ({ ...o, maps: next }))}
          >
            <RadioGroup
              legendText={tTournament("veto.mapMode")}
              name="mapMode"
              orientation="horizontal"
              value={values.mapMode as (typeof MAP_MODES)[number]}
              options={MAP_MODES.map((value) => ({
                value,
                label: tTournament(`veto.mapModes.${value}`),
              }))}
              onChange={(mapMode) => onChange({ mapMode })}
            />
            {values.mapMode === "veto" && (
              <>
                <TournamentVetoOrderField
                  values={values}
                  onChange={onChange}
                  maps={maps}
                  disabled={false}
                />
                <MapPoolField
                  tenant={POOL_KEY}
                  values={values}
                  onChange={onChange}
                  error={undefined}
                  maps={maps}
                  disabled={false}
                />
              </>
            )}
            {stepSecondsGroup(values) === "maps" && (
              <SecondsSelect
                labelText={tTournament("veto.seconds")}
                helperText={tTournament(
                  values.championPhase
                    ? "veto.secondsHintDraft"
                    : "veto.secondsHint",
                )}
                name="vetoSeconds"
                options={STEP_SECONDS}
                value={values.vetoSeconds}
                disabled={false}
                onChange={(vetoSeconds) => onChange({ vetoSeconds })}
              />
            )}
          </FormAccordionItem>
          <FormAccordionItem
            title={tTournament("champions")}
            summary={summary.champions(values)}
            group="champions"
            open={open.champions}
            onToggle={(next) => setOpen((o) => ({ ...o, champions: next }))}
          >
            <TournamentChampionFields
              values={values}
              onChange={onChange}
              errors={NO_ERRORS}
              disabled={false}
              repeatChoices={REPEAT_CHOICES}
            />
          </FormAccordionItem>
        </Accordion>
        {poolTooSmall && (
          <FormError>{t("poolTooSmall", { min: values.bestOf })}</FormError>
        )}
        {noPhases && <FormError>{t("noPhases")}</FormError>}
        {failure &&
          (failure.login ? (
            <Notice
              kind="warning"
              action={
                <ButtonLink
                  href={{ pathname: "/login", query: { return_to: "/veto" } }}
                  kind="tertiary"
                  size="sm"
                >
                  {tAuth("login")}
                </ButtonLink>
              }
            >
              {failure.message}
            </Notice>
          ) : (
            <FormError>{failure.message}</FormError>
          ))}
        <FormActions
          submitLabel={t("submit")}
          pending={pending}
          disabled={poolTooSmall || noPhases}
        />
      </Stack>
    </Form>
  );
}
