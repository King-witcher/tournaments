import {
  useCallback,
  useEffect,
  useMemo,
  useRef,
  useState,
  useTransition,
} from "react";

import { useRouter } from "@/i18n/navigation";
import { routing } from "@/i18n/routing";
import { callApi } from "@/shared/api/call";
import { api } from "@/shared/api/client";
import type { components } from "@/shared/api/schema";
import { useApiErrorMessage } from "@/shared/api/use-api-error-message";
import { useFormValidity } from "@/shared/hooks/use-form-validity";
import {
  controlSelector,
  type FieldErrors,
  translationFieldOf,
} from "../lib/form-errors";
import {
  type TranslationDraft,
  type TranslationValues,
  translationChange,
  translationFrom,
} from "../lib/form-values";
import { useSettingsDrafts } from "./use-settings-drafts";
import { useTournamentSettings } from "./use-tournament-settings";

type Schemas = components["schemas"];
type Locale = Schemas["Locale"];

type Failure = {
  message: string;
  /** O campo recusado e o idioma dele (o erro vai nele). */
  field?: keyof TranslationValues;
  locale?: Locale;
  /** O valor recusado: quando a pessoa muda o campo, o erro sai dele. */
  value?: string;
};

/** Sem erro: sempre o mesmo objeto, para o `memo` dos blocos. */
const NO_ERRORS: FieldErrors<keyof TranslationValues> = {};

/**
 * Outros idiomas, nas Configurações: uma tradução por idioma diferente do
 * original, com o que está salvo e o que a pessoa mudou por cima (guardado no
 * provider, para continuar lá depois de ir a outra seção). Salvar grava
 * cada tradução que mudou (PUT) e apaga a que foi removida, uma de cada vez;
 * a que a API recusa para no campo dela, com o foco.
 */
export function useLanguagesSection() {
  const { tenant, tournament, setSaved } = useTournamentSettings();
  const router = useRouter();
  const errorMessage = useApiErrorMessage();
  const { ref: formRef, canSubmit } = useFormValidity();
  const others = useMemo(
    () => routing.locales.filter((l) => l !== tournament.locale),
    [tournament.locale],
  );
  const { translationEdits: edits, setTranslationEdits: setEdits } =
    useSettingsDrafts();
  const [failure, setFailure] = useState<Failure>();
  const [done, setDone] = useState(false);
  const [pending, startTransition] = useTransition();
  const focused = useRef<Failure>(undefined);

  const draftOf = (l: Locale): TranslationDraft => ({
    ...translationFrom(tournament, l),
    ...edits[l],
  });

  const onChange = useCallback(
    (l: Locale, patch: Partial<TranslationDraft>) => {
      setEdits((all) => ({ ...all, [l]: { ...all[l], ...patch } }));
      setDone(false);
    },
    [setEdits],
  );

  const submit = () => {
    setFailure(undefined);
    setDone(false);
    startTransition(async () => {
      const path = { slug: tenant, tournament: tournament.slug };
      let translations = tournament.translations;
      for (const l of others) {
        const draft = draftOf(l);
        const change = translationChange(tournament, l, draft);
        if (change === undefined) continue;
        const params = { path: { ...path, locale: l } };
        if (change === "put") {
          const put = await callApi(
            api.PUT(
              "/tenants/{slug}/tournaments/{tournament}/translations/{locale}",
              {
                params,
                body: {
                  name: draft.name,
                  description: draft.description,
                  rules: draft.rules,
                },
              },
            ),
          );
          if (!put.ok) {
            const field = translationFieldOf(put.code);
            setFailure({
              message: errorMessage(put.code, put.params),
              field,
              locale: field && l,
              value: field && draft[field],
            });
            return;
          }
          translations = [
            ...translations.filter((x) => x.locale !== l),
            put.data,
          ];
        } else {
          const removed = await callApi(
            api.DELETE(
              "/tenants/{slug}/tournaments/{tournament}/translations/{locale}",
              { params },
            ),
          );
          if (!removed.ok) {
            setFailure({ message: errorMessage(removed.code, removed.params) });
            return;
          }
          translations = translations.filter((x) => x.locale !== l);
        }
        // O que já foi gravado vale, mesmo que um idioma seguinte falhe.
        setSaved({ ...tournament, translations });
        setEdits((all) => ({ ...all, [l]: undefined }));
      }
      setDone(true);
      router.refresh();
    });
  };

  useEffect(() => {
    if (!failure?.field || focused.current === failure) return;
    focused.current = failure;
    formRef.current
      ?.querySelector<HTMLElement>(
        controlSelector(failure.field, failure.locale),
      )
      ?.focus();
  }, [failure, formRef]);

  const live =
    failure?.field !== undefined &&
    failure.locale !== undefined &&
    draftOf(failure.locale)[failure.field] === failure.value
      ? failure
      : undefined;
  const liveErrors = useMemo<FieldErrors<keyof TranslationValues>>(
    () => (live?.field ? { [live.field]: live.message } : NO_ERRORS),
    [live],
  );

  return {
    formRef,
    canSubmit,
    /** O idioma original dos textos. */
    original: tournament.locale,
    /** Os idiomas que podem ter tradução. */
    others,
    draftOf,
    /** Muda a tradução de um idioma; a mesma função em todo render. */
    onChange,
    /** Os erros da tradução de `l`; mudam só quando o erro muda. */
    errorsOf: (l: Locale) => (live?.locale === l ? liveErrors : NO_ERRORS),
    /** O idioma tem tradução salva (remover a apaga ao salvar). */
    isSaved: (l: Locale) => tournament.translations.some((x) => x.locale === l),
    formError:
      failure && failure.field === undefined
        ? { message: failure.message }
        : undefined,
    done,
    /** Alguma tradução está diferente da salva (ou foi tirada). */
    dirty: others.some(
      (l) => translationChange(tournament, l, draftOf(l)) !== undefined,
    ),
    pending,
    submit,
  };
}
