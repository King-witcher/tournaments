import { useLocale } from "next-intl";
import {
  useCallback,
  useEffect,
  useMemo,
  useRef,
  useState,
  useTransition,
} from "react";

import { routing } from "@/i18n/routing";
import { callApi } from "@/shared/api/call";
import { api } from "@/shared/api/client";
import type { components } from "@/shared/api/schema";
import { useApiErrorMessage } from "@/shared/api/use-api-error-message";
import { useFormValidity } from "@/shared/hooks/use-form-validity";
import {
  controlSelector,
  type FieldErrors,
  tournamentFieldOf,
} from "../lib/form-errors";
import {
  createBody,
  minimalCreateBody,
  type TournamentValues,
  translationsFromSource,
  valuesFromSource,
} from "../lib/form-values";
import { rememberedPool } from "../lib/remembered-pool";

type Schemas = components["schemas"];
type Locale = Schemas["Locale"];

/** Os campos da criação curta. */
export type NewTournamentValues = Pick<
  TournamentValues,
  "name" | "slug" | "locale" | "mode" | "teamSize"
>;

/** Os campos que a criação mostra: o erro da API vai neles. */
const SHOWN = ["name", "slug", "teamSize"] as const;

type Failure = {
  message: string;
  field?: (typeof SHOWN)[number];
  /** O valor recusado: quando a pessoa muda o campo, o erro sai dele. */
  value?: unknown;
};

/** Sem erro: sempre o mesmo objeto, para o `memo` dos blocos. */
const NO_ERRORS: FieldErrors = {};

type Options = {
  tenant: string;
  /** O torneio de onde começar ao abrir ("Copiar torneio", `?from=`). */
  source?: Schemas["Tournament"];
  /** Os mapas ativos (o envio espera, se ainda estão chegando). */
  maps: () => Promise<Schemas["Map"][]>;
  /** Criado (sempre rascunho): para onde ir. */
  onCreated: (slug: string) => void;
};

/**
 * Criar torneio, curto e sempre rascunho: nome, endereço, modalidade e de
 * onde começar. Sem origem, manda só isso (e o pool lembrado da comunidade),
 * e o resto é o padrão da API. Com origem, a configuração dela inteira, como
 * na cópia de antes (`valuesFromSource`), com o nome e o endereço daqui; as
 * traduções copiadas levam o nome digitado, que a pessoa ajusta depois em
 * Outros idiomas. O resto se configura na aba Configurações.
 */
export function useNewTournament({ tenant, source, maps, onCreated }: Options) {
  const uiLocale = useLocale() as Locale;
  const errorMessage = useApiErrorMessage();
  const { ref: formRef, canSubmit } = useFormValidity();
  const [origin, setOrigin] = useState(source);
  const [values, setValues] = useState<NewTournamentValues>(() => ({
    name: "",
    slug: "",
    locale: source?.locale ?? uiLocale,
    mode: source?.mode ?? "duel",
    teamSize: source?.team_size ?? 1,
  }));
  const [failure, setFailure] = useState<Failure>();
  const [pending, startTransition] = useTransition();
  const focused = useRef<Failure>(undefined);

  const onChange = useCallback((patch: Partial<TournamentValues>) => {
    setValues((v) => ({
      name: patch.name ?? v.name,
      slug: patch.slug ?? v.slug,
      locale: v.locale,
      mode: patch.mode ?? v.mode,
      teamSize: patch.teamSize ?? v.teamSize,
    }));
  }, []);

  /**
   * Troca a origem. Com uma, a modalidade passa a ser a dela; sem nenhuma,
   * fica a escolhida. O nome e o endereço ficam como a pessoa os deixou.
   */
  const copyFrom = useCallback(
    (next: Schemas["Tournament"] | undefined) => {
      setOrigin(next);
      setValues((v) => ({
        ...v,
        locale: next?.locale ?? uiLocale,
        ...(next && { mode: next.mode, teamSize: next.team_size }),
      }));
    },
    [uiLocale],
  );

  const submit = () => {
    setFailure(undefined);
    startTransition(async () => {
      const active = await maps();
      const body = origin
        ? copied(origin, active, values)
        : minimalCreateBody(
            values,
            rememberedPool(
              tenant,
              active.map((m) => m.slug),
            ),
          );
      const created = await callApi(
        api.POST("/tenants/{slug}/tournaments", {
          params: { path: { slug: tenant } },
          body,
        }),
      );
      if (!created.ok) {
        const field = tournamentFieldOf(created.code);
        const shown = SHOWN.find((f) => f === field);
        // Erro de texto numa tradução copiada vem com o idioma: não é o campo daqui.
        const translation = created.params?.locale !== undefined;
        setFailure(
          shown && !translation
            ? {
                message: errorMessage(created.code, created.params),
                field: shown,
                value: values[shown],
              }
            : { message: errorMessage(created.code, created.params) },
        );
        return;
      }
      onCreated(created.data.slug);
    });
  };

  useEffect(() => {
    if (!failure?.field || focused.current === failure) return;
    focused.current = failure;
    formRef.current
      ?.querySelector<HTMLElement>(controlSelector(failure.field))
      ?.focus();
  }, [failure, formRef]);

  const live =
    failure?.field !== undefined && values[failure.field] === failure.value
      ? failure
      : undefined;
  const errors = useMemo<FieldErrors>(
    () => (live?.field ? { [live.field]: live.message } : NO_ERRORS),
    [live],
  );

  return {
    formRef,
    /** Nenhum campo recusa o que tem: "Criar" pode ser liberado. */
    canSubmit,
    values,
    /** Muda nome, endereço ou modalidade; a mesma função em todo render. */
    onChange,
    /** O torneio de onde começa, se há. */
    origin,
    /** Troca a origem; a mesma função em todo render. */
    copyFrom,
    errors,
    /** O erro que não é de um campo daqui. */
    formError:
      failure && failure.field === undefined ? failure.message : undefined,
    pending,
    submit,
  };
}

export type NewTournamentState = ReturnType<typeof useNewTournament>;

/**
 * O corpo da cópia: a configuração da origem (sem os mapas que saíram do
 * jogo), com o nome, o endereço e a modalidade daqui. As traduções da origem
 * vêm ligadas, com os textos dela e o nome digitado.
 */
function copied(
  origin: Schemas["Tournament"],
  maps: Schemas["Map"][],
  values: NewTournamentValues,
) {
  const base = valuesFromSource(origin, maps);
  const others = routing.locales.filter((l) => l !== origin.locale);
  const translations = Object.fromEntries(
    Object.entries(translationsFromSource(origin, others)).map(([l, draft]) => [
      l,
      draft && { ...draft, name: values.name },
    ]),
  );
  return createBody({ ...base, ...values }, translations);
}
