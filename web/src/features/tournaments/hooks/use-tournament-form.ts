import { useLocale } from "next-intl";
import {
  startTransition,
  useActionState,
  useEffect,
  useEffectEvent,
  useRef,
  useState,
} from "react";

import { useRouter } from "@/i18n/navigation";
import { routing } from "@/i18n/routing";
import { type ApiErrorParams, callApi } from "@/shared/api/call";
import { api } from "@/shared/api/client";
import type { components } from "@/shared/api/schema";
import { useApiErrorMessage } from "@/shared/api/use-api-error-message";
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
  updateBody,
  valuesFrom,
} from "../lib/form-values";
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

type Options = {
  /**
   * Mostra o que esconde um controle (o acordeão, a aba, o idioma), para o
   * foco chegar a ele quando é recusado.
   */
  reveal: (control: HTMLElement) => void;
};

/**
 * Estado e envio do formulário de torneio. Criar faz um POST só, com as
 * traduções ligadas e, em "Criar", já publicando (a API grava tudo ou nada).
 * Editar faz o PATCH e, depois, grava cada tradução que mudou (PUT) ou apaga
 * a que foi desligada, tudo no mesmo "Salvar". Deu certo, vai para a página
 * do torneio. O erro de um campo vai para ele, que é mostrado (acordeão, aba)
 * e recebe o foco; o resto é do formulário.
 */
export function useTournamentForm(
  tenant: string,
  tournament: Schemas["Tournament"] | undefined,
  { reveal }: Options,
) {
  const uiLocale = useLocale() as Locale;
  const router = useRouter();
  const errorMessage = useApiErrorMessage();
  const formRef = useRef<HTMLFormElement>(null);
  // Na criação, o original é o idioma da interface; a outra aba é a tradução.
  const original = tournament?.locale ?? uiLocale;
  const others = routing.locales.filter((l) => l !== original);
  const [values, setValues] = useState(() =>
    tournament ? valuesFrom(tournament) : defaultValues(original),
  );
  const [translations, setTranslations] = useState<Translations>(() =>
    Object.fromEntries(
      others.map((l) => [
        l,
        tournament ? translationFrom(tournament, l) : EMPTY_TRANSLATION,
      ]),
    ),
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
    router.push(`/${tenant}/${tournament.slug}`);
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
  const errorsOf = (at?: Locale): FieldErrors =>
    live?.field && live.locale === at ? { [live.field]: live.message } : {};

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
    values,
    onChange: (patch: Partial<TournamentValues>) =>
      setValues((v) => ({ ...v, ...patch })),
    /** O idioma original dos textos. */
    original,
    /** Os idiomas que podem ter tradução (os diferentes do original). */
    others,
    translations,
    /** A tradução de `l` (vazia e desligada se não existe). */
    translationOf: (l: Locale): TranslationDraft =>
      translations[l] ?? EMPTY_TRANSLATION,
    onTranslationChange: (l: Locale, patch: Partial<TranslationDraft>) =>
      setTranslations((all) => ({
        ...all,
        [l]: { ...EMPTY_TRANSLATION, ...all[l], ...patch },
      })),
    setupLocked,
    check,
    /** Envia (depois de `check`). */
    submit: (intent: SubmitIntent) => {
      setChecked(undefined);
      startTransition(() => dispatch(intent));
    },
    pending,
    errors: errorsOf(),
    translationErrors: (l: Locale) =>
      errorsOf(l) as FieldErrors<keyof TranslationValues>,
    formError:
      failure && failure.field === undefined ? failure.message : undefined,
  };
}

/** O formulário de torneio: o que `useTournamentForm` devolve. */
export type TournamentFormState = ReturnType<typeof useTournamentForm>;
