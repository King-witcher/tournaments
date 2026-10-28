import { useLocale } from "next-intl";
import { useActionState, useEffect, useRef, useState } from "react";

import { useRouter } from "@/i18n/navigation";
import { routing } from "@/i18n/routing";
import { type ApiErrorParams, callApi } from "@/shared/api/call";
import { api } from "@/shared/api/client";
import type { components } from "@/shared/api/schema";
import { useApiErrorMessage } from "@/shared/api/use-api-error-message";
import {
  controlName,
  type FieldErrors,
  type TournamentField,
  tournamentFieldOf,
  translationFieldOf,
} from "../lib/form-errors";
import {
  createBody,
  defaultValues,
  EMPTY_TRANSLATION,
  type TournamentValues,
  type TranslationValues,
  translationChanged,
  translationValuesFrom,
  updateBody,
  valuesFrom,
} from "../lib/form-values";
import { setupEditable } from "../lib/status";

type Schemas = components["schemas"];
type Locale = Schemas["Locale"];
type Translations = Partial<Record<Locale, TranslationValues>>;

/** O que a API recusou no último envio. */
type Failure = {
  message: string;
  /** O campo recusado; sem ele, o erro é do formulário inteiro. */
  field?: TournamentField;
  /** O campo é da tradução deste idioma. */
  locale?: Locale;
  /** O valor recusado: quando a pessoa muda o campo, o erro sai dele. */
  value?: unknown;
};

/**
 * Estado e envio do formulário de torneio. Criar faz o POST do rascunho;
 * editar faz o PATCH e, depois, grava cada tradução que mudou (PUT), tudo no
 * mesmo "Salvar". Deu certo, vai para a página do torneio. O erro de um
 * campo vai para ele (e o foco também); o resto é do formulário.
 */
export function useTournamentForm(
  tenant: string,
  tournament?: Schemas["Tournament"],
) {
  const locale = useLocale();
  const router = useRouter();
  const errorMessage = useApiErrorMessage();
  const formRef = useRef<HTMLFormElement>(null);
  const others = tournament
    ? routing.locales.filter((l) => l !== tournament.locale)
    : [];
  const [values, setValues] = useState(() =>
    tournament ? valuesFrom(tournament) : defaultValues(locale),
  );
  const [translations, setTranslations] = useState<Translations>(() =>
    tournament
      ? Object.fromEntries(
          others.map((l) => [l, translationValuesFrom(tournament, l)]),
        )
      : {},
  );
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

  const [failure, submit, pending] = useActionState<Failure | undefined>(
    async () => {
      if (!tournament) {
        const created = await callApi(
          api.POST("/tenants/{slug}/tournaments", {
            params: { path: { slug: tenant } },
            body: createBody(values),
          }),
        );
        if (!created.ok) return fail(created.code, created.params);
        router.push(`/${tenant}/${created.data.slug}`);
        router.refresh();
        return undefined;
      }
      const path = { slug: tenant, tournament: tournament.slug };
      const saved = await callApi(
        api.PATCH("/tenants/{slug}/tournaments/{tournament}", {
          params: { path },
          body: updateBody(values, setupLocked),
        }),
      );
      if (!saved.ok) return fail(saved.code, saved.params);
      for (const l of others) {
        const body = translations[l];
        if (!body || !translationChanged(tournament, l, body)) continue;
        const put = await callApi(
          api.PUT(
            "/tenants/{slug}/tournaments/{tournament}/translations/{locale}",
            { params: { path: { ...path, locale: l } }, body },
          ),
        );
        if (!put.ok) return fail(put.code, put.params, l);
      }
      router.push(`/${tenant}/${tournament.slug}`);
      router.refresh();
      return undefined;
    },
    undefined,
  );

  // Erro num campo: o foco vai para ele, e o leitor de tela lê o rótulo e o erro.
  useEffect(() => {
    if (failure?.field === undefined) return;
    formRef.current
      ?.querySelector<HTMLElement>(
        `[name="${controlName(failure.field, failure.locale)}"]`,
      )
      ?.focus();
  }, [failure]);

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
  const errorsOf = (at?: Locale): FieldErrors =>
    live?.field && live.locale === at ? { [live.field]: live.message } : {};

  return {
    formRef,
    values,
    onChange: (patch: Partial<TournamentValues>) =>
      setValues((v) => ({ ...v, ...patch })),
    /** Os idiomas que podem ter tradução (os diferentes do original). */
    others,
    /** Os campos da tradução de `l` (vazios se ela não existe). */
    translationOf: (l: Locale) => translations[l] ?? EMPTY_TRANSLATION,
    onTranslationChange: (l: Locale, patch: Partial<TranslationValues>) =>
      setTranslations((all) => ({
        ...all,
        [l]: { ...EMPTY_TRANSLATION, ...all[l], ...patch },
      })),
    /** Depois de remover a tradução, os campos dela voltam a ficar vazios. */
    clearTranslation: (l: Locale) =>
      setTranslations((all) => ({ ...all, [l]: EMPTY_TRANSLATION })),
    setupLocked,
    submit,
    pending,
    errors: errorsOf(),
    translationErrors: (l: Locale) =>
      errorsOf(l) as FieldErrors<keyof TranslationValues>,
    formError:
      failure && failure.field === undefined ? failure.message : undefined,
  };
}
