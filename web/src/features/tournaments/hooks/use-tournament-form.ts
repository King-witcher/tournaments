import { useLocale } from "next-intl";
import {
  startTransition,
  useActionState,
  useCallback,
  useEffect,
  useEffectEvent,
  useMemo,
  useState,
} from "react";

import { useRealtimeHub } from "@/features/realtime/hooks/use-realtime-hub";
import { useRouter } from "@/i18n/navigation";
import { routing } from "@/i18n/routing";
import { type ApiErrorParams, callApi } from "@/shared/api/call";
import { api } from "@/shared/api/client";
import type { components } from "@/shared/api/schema";
import { useApiErrorMessage } from "@/shared/api/use-api-error-message";
import { useFormValidity } from "@/shared/hooks/use-form-validity";
import { focusWhenShown } from "@/shared/lib/focus";
import {
  controlName,
  type FieldErrors,
  type TournamentField,
  tournamentFieldOf,
  translationFieldOf,
} from "../lib/form-errors";
import { firstInvalid } from "../lib/form-groups";
import {
  createBody,
  defaultValues,
  EMPTY_TRANSLATION,
  type TournamentValues,
  type TranslationDraft,
  type Translations,
  type TranslationValues,
  translationChange,
  translationFrom,
  translationsFromSource,
  updateBody,
  valuesFrom,
  valuesFromSource,
} from "../lib/form-values";
import { rememberedPool } from "../lib/remembered-pool";
import { setupEditable } from "../lib/status";

type Schemas = components["schemas"];
type Locale = Schemas["Locale"];

/** Criar como rascunho, criar e publicar, ou salvar a edição. */
export type SubmitIntent = "draft" | "publish" | "save";

/** O que a API (ou a conferência antes de enviar) recusou. */
type Failure = {
  message: string;
  /** O campo recusado; sem ele, o erro é do formulário inteiro. */
  field?: TournamentField;
  /** O campo é da tradução deste idioma. */
  locale?: Locale;
  /** O valor recusado: quando a pessoa muda o campo, o erro sai dele. */
  value?: unknown;
};

/** Sem erro: sempre o mesmo objeto, para o `memo` dos blocos. */
const NO_ERRORS: FieldErrors = {};

/** Os idiomas que podem ter tradução: os diferentes do original. */
const otherLocales = (original: Locale) =>
  routing.locales.filter((l) => l !== original);

type Options = {
  /**
   * Mostra o que esconde um controle (o acordeão ou a aba), para o
   * foco chegar a ele quando é recusado.
   */
  reveal: (control: HTMLElement) => void;
  /** Na criação: o torneio copiado ao abrir (`valuesFromSource`). */
  source?: Schemas["Tournament"];
  /** Na criação: os mapas ativos, que limitam o pool copiado. */
  maps?: Schemas["Map"][];
  /**
   * Na criação: o que fazer com o torneio criado (`slug`) antes de ir para
   * ele (mandar a capa). Falso fica na página.
   */
  afterCreate?: (slug: string) => Promise<boolean>;
};

/**
 * Estado e envio do formulário de torneio. A criação começa do padrão ou da
 * cópia de outro torneio (`source`, trocável por `copyFrom`). Criar faz um
 * POST só, com as traduções ligadas e, em "Criar", já publicando (a API grava
 * tudo ou nada).
 * Editar faz o PATCH e, depois, grava cada tradução que mudou (PUT) ou apaga
 * a que foi desligada, tudo no mesmo "Salvar". Deu certo, vai para a página
 * do torneio, no endereço novo se ele mudou (as outras abas vão sozinhas, e
 * esta não precisa do aviso: `originHeaders`). O erro de um campo vai para ele,
 * que é mostrado (acordeão ou aba) e recebe o foco; o resto é do formulário.
 */
export function useTournamentForm(
  tenant: string,
  tournament: Schemas["Tournament"] | undefined,
  { reveal, source, maps, afterCreate }: Options,
) {
  const uiLocale = useLocale() as Locale;
  const router = useRouter();
  const hub = useRealtimeHub();
  const errorMessage = useApiErrorMessage();
  // O endereço gravado: uma tradução recusada depois de o PATCH trocar o
  // endereço deixa o formulário na tela, e o próximo envio já vai pelo novo.
  const [savedSlug, setSavedSlug] = useState(tournament?.slug);
  // `canSubmit`: todos os campos válidos (os de `shared/ui` declaram o que cobram).
  const { ref: formRef, canSubmit } = useFormValidity();
  const [values, setValues] = useState(() =>
    tournament
      ? valuesFrom(tournament)
      : source
        ? valuesFromSource(source, maps ?? [])
        : defaultValues(uiLocale),
  );
  // O original é o idioma do torneio ou o da origem copiada (as traduções
  // dela continuam traduções); sem nenhum, o da interface.
  const original = values.locale;
  const others = useMemo(() => otherLocales(original), [original]);
  const [translations, setTranslations] = useState<Translations>(() =>
    tournament
      ? Object.fromEntries(
          others.map((l) => [l, translationFrom(tournament, l)]),
        )
      : translationsFromSource(source, others),
  );
  /** O que a conferência antes de enviar recusou (a data para publicar). */
  const [checked, setChecked] = useState<Failure>();
  const setupLocked = tournament ? !setupEditable(tournament.status) : false;

  const fail = (
    code: string,
    params?: ApiErrorParams,
    at?: Locale,
  ): Failure => {
    const field = at ? translationFieldOf(code) : tournamentFieldOf(code);
    const value =
      field === undefined
        ? undefined
        : at
          ? translations[at]?.[field as keyof TranslationValues]
          : values[field];
    return { message: errorMessage(code, params), field, locale: at, value };
  };

  const [sent, dispatch, pending] = useActionState<
    Failure | undefined,
    SubmitIntent
  >(async (_previous, intent) => {
    if (!tournament) {
      const created = await callApi(
        api.POST("/tenants/{slug}/tournaments", {
          params: { path: { slug: tenant } },
          body: createBody(values, translations, intent === "publish"),
        }),
      );
      if (!created.ok) {
        // Erro de texto numa tradução vem com o idioma dela.
        const at = created.params?.locale;
        return fail(
          created.code,
          created.params,
          typeof at === "string" ? (at as Locale) : undefined,
        );
      }
      const slug = created.data.slug;
      if (afterCreate && !(await afterCreate(slug))) return undefined;
      router.push(`/${tenant}/${slug}`);
      router.refresh();
      return undefined;
    }
    const saved = await callApi(
      api.PATCH("/tenants/{slug}/tournaments/{tournament}", {
        params: {
          path: { slug: tenant, tournament: savedSlug ?? tournament.slug },
        },
        body: updateBody(values, tournament.status),
        headers: hub?.originHeaders(),
      }),
    );
    if (!saved.ok) return fail(saved.code, saved.params);
    setSavedSlug(saved.data.slug);
    const path = { slug: tenant, tournament: saved.data.slug };
    for (const l of others) {
      const draft = translations[l] ?? EMPTY_TRANSLATION;
      const change = translationChange(tournament, l, draft);
      if (change === undefined) continue;
      const params = { path: { ...path, locale: l } };
      const done = await callApi(
        change === "put"
          ? api.PUT(
              "/tenants/{slug}/tournaments/{tournament}/translations/{locale}",
              {
                params,
                body: {
                  name: draft.name,
                  description: draft.description,
                  rules: draft.rules,
                },
              },
            )
          : api.DELETE(
              "/tenants/{slug}/tournaments/{tournament}/translations/{locale}",
              { params },
            ),
      );
      if (!done.ok) return fail(done.code, done.params, l);
    }
    router.push(`/${tenant}/${saved.data.slug}`);
    router.refresh();
    return undefined;
  }, undefined);

  // A conferência local vale até o próximo envio, que a limpa.
  const failure = checked ?? sent;

  // Erro num campo: mostra o que o esconde e leva o foco até ele (pelo
  // `name`); o leitor de tela lê o rótulo e o erro.
  const showFailure = useEffectEvent((f: Failure) => {
    if (!f.field) return;
    const control = formRef.current?.querySelector<HTMLElement>(
      `[name="${controlName(f.field, f.locale)}"]`,
    );
    if (!control) return;
    reveal(control);
    focusWhenShown(control);
  });
  // Um efeito por origem: limpar a conferência não pode levar o foco de
  // volta ao erro antigo da API.
  useEffect(() => {
    if (sent) showFailure(sent);
  }, [sent]);
  useEffect(() => {
    if (checked) showFailure(checked);
  }, [checked]);

  const current = (f: Failure) =>
    f.field === undefined
      ? undefined
      : f.locale
        ? translations[f.locale]?.[f.field as keyof TranslationValues]
        : values[f.field];
  const live =
    failure?.field !== undefined && current(failure) === failure.value
      ? failure
      : undefined;
  // Os erros mudam de objeto só quando o erro muda: os blocos do formulário
  // (com `memo`) não renderizam de novo a cada tecla por causa deles.
  const liveErrors = useMemo<FieldErrors>(
    () => (live?.field ? { [live.field]: live.message } : NO_ERRORS),
    [live],
  );
  const errorsOf = useCallback(
    (at?: Locale) => (live?.locale === at ? liveErrors : NO_ERRORS),
    [live, liveErrors],
  );
  const onChange = useCallback(
    (patch: Partial<TournamentValues>) =>
      setValues((v) => ({ ...v, ...patch })),
    [],
  );
  const onTranslationChange = useCallback(
    (l: Locale, patch: Partial<TranslationDraft>) =>
      setTranslations((all) => ({
        ...all,
        [l]: { ...EMPTY_TRANSLATION, ...all[l], ...patch },
      })),
    [],
  );
  const translationErrors = useCallback(
    (l: Locale) => errorsOf(l) as FieldErrors<keyof TranslationValues>,
    [errorsOf],
  );
  const copyFrom = useCallback(
    (next: Schemas["Tournament"] | undefined) => {
      const active = maps ?? [];
      const start = next
        ? valuesFromSource(next, active)
        : {
            ...defaultValues(uiLocale),
            mapPool: rememberedPool(
              tenant,
              active.map((m) => m.slug),
            ),
          };
      setValues((v) => ({
        ...start,
        name: v.name,
        slug: v.slug,
        startsAt: v.startsAt,
      }));
      const locales = otherLocales(start.locale);
      const copied = translationsFromSource(next, locales);
      // O nome de cada tradução também é da edição: fica o que foi digitado.
      setTranslations((all) =>
        Object.fromEntries(
          locales.map((l) => [
            l,
            { ...(copied[l] ?? EMPTY_TRANSLATION), name: all[l]?.name ?? "" },
          ]),
        ),
      );
    },
    [tenant, uiLocale, maps],
  );

  /**
   * Confere antes de enviar: os campos (a validação nativa, com o grupo do
   * primeiro recusado aberto). Recusado,
   * o foco vai para o campo.
   */
  const check = () => {
    const form = formRef.current;
    if (!form) return false;
    const invalid = firstInvalid(form);
    if (invalid) {
      reveal(invalid);
      focusWhenShown(invalid, () => invalid.reportValidity());
      return false;
    }
    setChecked(undefined);
    return true;
  };

  return {
    formRef,
    /** Nenhum campo recusa o que tem: o envio pode ser liberado. */
    canSubmit,
    values,
    /** Muda campos do torneio; a mesma função em todo render. */
    onChange,
    /** O idioma original dos textos. */
    original,
    /** Os idiomas que podem ter tradução (os diferentes do original). */
    others,
    translations,
    /** A tradução de `l` (vazia e desligada se não existe). */
    translationOf: (l: Locale): TranslationDraft =>
      translations[l] ?? EMPTY_TRANSLATION,
    /** Muda a tradução de um idioma; a mesma função em todo render. */
    onTranslationChange,
    /**
     * Na criação, troca a origem (`undefined`: o padrão, com o pool
     * lembrado). Tudo passa a ser o dela, menos o que identifica a edição e
     * a pessoa pode já ter digitado: nome, endereço, início e o nome das
     * traduções. A mesma função em todo render.
     */
    copyFrom,
    setupLocked,
    /** Na edição, o endereço gravado agora. */
    savedSlug,
    /** Na edição: salvar troca o endereço, o que pede a confirmação. */
    movesAddress: savedSlug !== undefined && values.slug !== savedSlug,
    check,
    /** Envia (depois de `check`). */
    submit: (intent: SubmitIntent) => {
      setChecked(undefined);
      startTransition(() => dispatch(intent));
    },
    pending,
    errors: errorsOf(),
    /** Os erros da tradução de um idioma; muda só quando o erro muda. */
    translationErrors,
    formError:
      failure && failure.field === undefined ? failure.message : undefined,
  };
}

/** O formulário de torneio: o que `useTournamentForm` devolve. */
export type TournamentFormState = ReturnType<typeof useTournamentForm>;
