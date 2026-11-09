"use client";

import {
  type ReactNode,
  useCallback,
  useEffect,
  useMemo,
  useRef,
  useState,
} from "react";

import { useImageDraft } from "@/features/images/hooks/use-image-draft";
import { useRouter } from "@/i18n/navigation";
import { routing } from "@/i18n/routing";
import type { components } from "@/shared/api/schema";
import {
  SettingsDraftsContext,
  type TranslationEdits,
  type ValuesEdits,
} from "../hooks/use-settings-drafts";
import { TournamentSettingsContext } from "../hooks/use-tournament-settings";
import {
  translationChange,
  translationFrom,
  valuesFrom,
} from "../lib/form-values";
import {
  changed,
  SECTION_KEYS,
  SETTINGS_SECTIONS,
  type SettingsSection,
  sectionOfHash,
  type ValuesSection,
} from "../lib/settings-sections";
import { settingsHref } from "../lib/tabs";
import { SettingsLeaveGuard } from "./settings-leave-guard";

type Tournament = components["schemas"]["Tournament"];

type Props = {
  tenant: string;
  /** O torneio que o layout das Configurações buscou. */
  tournament: Tournament;
  children: ReactNode;
};

/** Sem mudança: sempre o mesmo objeto, para o `memo` dos blocos. */
const NO_EDITS: ValuesEdits = {};

/**
 * O estado das Configurações do torneio, no `layout.tsx` delas, que não
 * remonta ao trocar de seção. O torneio salvo, para todas as seções: cada uma
 * manda o que está salvo mais os campos dela. Depois de salvar, vale a
 * resposta da API até a página trazer o torneio de novo (`router.refresh` ou
 * o tempo real), para outra seção salva logo em seguida não mandar o valor
 * antigo, e para o que depende do salvo de outra seção (as opções de campeões
 * do TDM, as séries que pedem mais mapas que o pool) mudar sem recarregar.
 * Também guarda o que a pessoa mudou em cada seção e não salvou, para a
 * mudança continuar lá ao ir a outra seção e voltar, e pergunta antes de sair
 * das Configurações com alguma (`SettingsLeaveGuard`). O endereço antigo com
 * `#maps` vai à página de Mapas.
 */
export function TournamentSettingsProvider({
  tenant,
  tournament,
  children,
}: Props) {
  const router = useRouter();
  const [fromPage, setFromPage] = useState(tournament);
  const [saved, setSaved] = useState(tournament);
  if (fromPage !== tournament) {
    setFromPage(tournament);
    setSaved(tournament);
  }
  const value = useMemo(
    () => ({ tenant, tournament: saved, setSaved }),
    [tenant, saved],
  );

  const [edits, setAllEdits] = useState<
    Partial<Record<ValuesSection, ValuesEdits>>
  >({});
  const [translationEdits, setTranslationEdits] = useState<TranslationEdits>(
    {},
  );
  const setEdits = useCallback(
    (section: ValuesSection, update: (e: ValuesEdits) => ValuesEdits) =>
      setAllEdits((all) => ({
        ...all,
        [section]: update(all[section] ?? NO_EDITS),
      })),
    [],
  );

  const coverTarget = useMemo(
    () => ({ kind: "cover" as const, tenant, tournament: saved.slug }),
    [tenant, saved.slug],
  );
  const cover = useImageDraft(coverTarget);
  const coverDirty = cover.dirty;

  const dirty = useMemo(() => {
    const values = valuesFrom(saved);
    return SETTINGS_SECTIONS.filter((section) => {
      if (section === "general" && coverDirty) return true;
      if (section === "languages") {
        return routing.locales.some(
          (l) =>
            l !== saved.locale &&
            translationChange(saved, l, {
              ...translationFrom(saved, l),
              ...translationEdits[l],
            }) !== undefined,
        );
      }
      const mine = edits[section];
      if (!mine) return false;
      const next = { ...values, ...mine };
      return SECTION_KEYS[section].some((k) => changed(values, next, k));
    });
  }, [saved, edits, translationEdits, coverDirty]);

  // O pedido de foco do "Ir para", que a seção de destino atende ao montar.
  const arrival = useRef<SettingsSection>(undefined);
  const base = `/${tenant}/${saved.slug}`;
  const goTo = useCallback(
    (section: SettingsSection) => {
      arrival.current = section;
      router.push(settingsHref(base, section));
    },
    [router, base],
  );
  const takeArrival = useCallback((section: SettingsSection) => {
    if (arrival.current !== section) return false;
    arrival.current = undefined;
    return true;
  }, []);
  // O endereço de antes das páginas por seção (`settings#maps`).
  useEffect(() => {
    const section = sectionOfHash(window.location.hash);
    if (section && section !== "general") {
      arrival.current = section;
      router.replace(settingsHref(base, section));
    }
  }, [router, base]);

  const drafts = useMemo(
    () => ({
      editsOf: (s: ValuesSection) => edits[s] ?? NO_EDITS,
      setEdits,
      translationEdits,
      setTranslationEdits,
      cover,
      dirty,
      goTo,
      takeArrival,
    }),
    [edits, setEdits, translationEdits, cover, dirty, goTo, takeArrival],
  );

  return (
    <TournamentSettingsContext value={value}>
      <SettingsDraftsContext value={drafts}>
        {children}
        <SettingsLeaveGuard
          settingsPath={settingsHref(base, "general")}
          dirty={dirty}
        />
      </SettingsDraftsContext>
    </TournamentSettingsContext>
  );
}
