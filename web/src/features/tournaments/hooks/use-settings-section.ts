import {
  useCallback,
  useEffect,
  useMemo,
  useRef,
  useState,
  useTransition,
} from "react";

import { useRealtimeHub } from "@/features/realtime/hooks/use-realtime-hub";
import { useRouter } from "@/i18n/navigation";
import { callApi } from "@/shared/api/call";
import { api } from "@/shared/api/client";
import { useApiErrorMessage } from "@/shared/api/use-api-error-message";
import { useFormValidity } from "@/shared/hooks/use-form-validity";
import { focusWhenShown } from "@/shared/lib/focus";
import {
  controlSelector,
  type FieldErrors,
  isStageField,
  type TournamentField,
  tournamentFieldOf,
} from "../lib/form-errors";
import {
  fieldValue,
  type TournamentValues,
  updateBody,
  valuesFrom,
} from "../lib/form-values";
import {
  changed,
  SECTION_KEYS,
  type SettingsSection,
  sectionOf,
  type ValuesSection,
} from "../lib/settings-sections";
import { settingsHref } from "../lib/tabs";
import { useSettingsDrafts } from "./use-settings-drafts";
import { useTournamentSettings } from "./use-tournament-settings";

/** O que a API recusou no último envio. */
export type SectionFailure = {
  message: string;
  /** O campo recusado, quando ele está nesta seção (o erro vai nele). */
  field?: TournamentField;
  /** O valor recusado: quando a pessoa muda o campo, o erro sai dele. */
  value?: unknown;
  /** Outra seção, onde se corrige o que a API recusou. */
  fixIn?: SettingsSection;
};

/** Sem erro: sempre o mesmo objeto, para o `memo` dos blocos. */
const NO_ERRORS: FieldErrors = {};

/**
 * Uma seção das Configurações, que salva sozinha: os valores são os salvos
 * com o que a pessoa mudou aqui por cima, guardado no provider para continuar
 * lá depois de ir a outra seção e voltar. Salvar manda o torneio salvo com os
 * campos desta seção (a API compara e só aplica o que mudou); o que foi
 * mexido em outra seção e não salvo não vai junto. O erro de um campo desta
 * seção vai para ele, que recebe o foco; o que nasce de outra seção (as
 * séries daqui pedem mais mapas que o pool salvo) fica no fim desta, com a
 * seção onde corrigir. Salvo, a página recarrega e, se o endereço mudou, vai
 * para o novo (as outras abas vão sozinhas; esta não recebe o aviso:
 * `originHeaders`).
 *
 * Geral leva também a capa pendente (`cover`, do provider), mandada antes dos
 * campos: o endereço de agora ainda vale para ela, e o que se troca e o que
 * dá errado não dependem um do outro. A imagem que falha fica pendente, com
 * o erro nela, e os campos seguem, menos se trocariam o endereço: a página
 * nova perderia a imagem e o erro. O que já foi aceito não volta no próximo
 * Salvar: só a imagem mudada não manda os campos, e a capa aceita sai da
 * pendência antes do envio deles.
 */
export function useSettingsSection(section: ValuesSection) {
  const { tenant, tournament, setSaved } = useTournamentSettings();
  const router = useRouter();
  const hub = useRealtimeHub();
  const errorMessage = useApiErrorMessage();
  const { ref: formRef, canSubmit } = useFormValidity();
  const keys = SECTION_KEYS[section] as readonly (keyof TournamentValues)[];
  const saved = useMemo(() => valuesFrom(tournament), [tournament]);
  const { editsOf, setEdits: setSectionEdits, cover } = useSettingsDrafts();
  const image = section === "general" ? cover : undefined;
  const edits = editsOf(section);
  const setEdits = useCallback(
    (update: (e: Partial<TournamentValues>) => Partial<TournamentValues>) =>
      setSectionEdits(section, update),
    [setSectionEdits, section],
  );
  const values = useMemo(() => ({ ...saved, ...edits }), [saved, edits]);
  const [failure, setFailure] = useState<SectionFailure>();
  const [done, setDone] = useState(false);
  const [pending, startTransition] = useTransition();
  // Para o foco ir ao campo só uma vez por recusa.
  const focused = useRef<SectionFailure>(undefined);

  const onChange = useCallback(
    (patch: Partial<TournamentValues>) => {
      setEdits((e) => ({ ...e, ...patch }));
      setDone(false);
    },
    [setEdits],
  );

  const fieldsDirty = keys.some((k) => changed(saved, values, k));

  const submit = () => {
    setFailure(undefined);
    setDone(false);
    startTransition(async () => {
      const mine = Object.fromEntries(keys.map((k) => [k, values[k]]));
      const sentImage = image?.dirty ?? false;
      const imageSaved = image ? await image.apply() : true;
      if (!imageSaved && values.slug !== tournament.slug) return;
      if (sentImage && !fieldsDirty) {
        if (imageSaved) setDone(true);
        router.refresh();
        return;
      }
      const outcome = await callApi(
        api.PATCH("/tenants/{slug}/tournaments/{tournament}", {
          params: { path: { slug: tenant, tournament: tournament.slug } },
          body: updateBody({ ...saved, ...mine }, tournament.status),
          headers: hub?.originHeaders(),
        }),
      );
      if (!outcome.ok) {
        const field = tournamentFieldOf(outcome.code, outcome.params);
        const message = errorMessage(outcome.code, outcome.params);
        const here =
          field !== undefined &&
          (isStageField(field) ? section === "format" : keys.includes(field)) &&
          formRef.current?.querySelector(controlSelector(field)) != null;
        setFailure(
          here
            ? { message, field, value: fieldValue(values, field) }
            : {
                message,
                fixIn:
                  field === undefined ? undefined : sectionOf(field, saved),
              },
        );
        if (sentImage) router.refresh();
        return;
      }
      setSaved(outcome.data);
      // O que mudou enquanto salvava continua por cima do salvo.
      setEdits((e) =>
        Object.fromEntries(
          Object.entries(e).filter(
            ([k, v]) => !Object.is(v, mine[k as keyof typeof mine]),
          ),
        ),
      );
      setDone(imageSaved);
      if (outcome.data.slug !== tournament.slug) {
        // O endereço novo troca a árvore (o que não foi salvo nas outras
        // seções se perde).
        router.replace(
          settingsHref(`/${tenant}/${outcome.data.slug}`, section),
        );
      }
      router.refresh();
    });
  };

  // O campo recusado recebe o foco: o leitor de tela lê o rótulo e o erro.
  useEffect(() => {
    if (!failure?.field || focused.current === failure) return;
    focused.current = failure;
    // A fase de outra aba só aparece depois: espera ficar à vista.
    const control = formRef.current?.querySelector<HTMLElement>(
      controlSelector(failure.field),
    );
    if (control) focusWhenShown(control);
  }, [failure, formRef]);

  const live =
    failure?.field !== undefined &&
    Object.is(fieldValue(values, failure.field), failure.value)
      ? failure
      : undefined;
  const errors = useMemo<FieldErrors>(
    () => (live?.field ? { [live.field]: live.message } : NO_ERRORS),
    [live],
  );
  const dirty = fieldsDirty || (image?.dirty ?? false);

  return {
    formRef,
    /** Nenhum campo recusa o que tem: o Salvar pode ser liberado. */
    canSubmit,
    /** O que está salvo no torneio. */
    saved,
    /** O salvo com o que a pessoa mudou nesta seção. */
    values,
    /** Muda campos desta seção; a mesma função em todo render. */
    onChange,
    /** Os erros dos campos desta seção; muda só quando o erro muda. */
    errors,
    /** O erro que não é de um campo desta seção, com onde corrigir. */
    formError: failure && failure.field === undefined ? failure : undefined,
    /** Acabou de salvar e nada mudou desde então. */
    done: done && !image?.dirty,
    /** Algum campo desta seção está diferente do salvo, ou a capa pendente. */
    dirty,
    pending,
    submit,
  };
}

export type SettingsSectionState = ReturnType<typeof useSettingsSection>;
