"use client";

import {
  type ReactNode,
  useCallback,
  useEffect,
  useMemo,
  useState,
} from "react";

import type { components } from "@/shared/api/schema";
import { SettingsItemsContext } from "../hooks/use-settings-items";
import { TournamentSettingsContext } from "../hooks/use-tournament-settings";
import {
  type SettingsSection,
  sectionItemId,
  sectionOfHash,
} from "../lib/settings-sections";

type Tournament = components["schemas"]["Tournament"];

type Props = {
  tenant: string;
  /** O torneio que a página buscou. */
  tournament: Tournament;
  children: ReactNode;
};

/**
 * O torneio salvo, para todas as seções da aba Configurações: cada uma manda o
 * que está salvo mais os campos dela. Depois de salvar, vale a resposta da API
 * até a página trazer o torneio de novo (`router.refresh` ou o tempo real),
 * para outra seção salva logo em seguida não mandar o valor antigo, e para o
 * que depende do salvo de outra seção (as opções de campeões do TDM, as
 * séries que pedem mais mapas que o pool) mudar sem recarregar. Também guarda
 * quais itens do acordeão estão abertos: todos fechados ao abrir a página,
 * menos o que o `#` do endereço nomeia (`#maps`).
 */
export function TournamentSettingsProvider({
  tenant,
  tournament,
  children,
}: Props) {
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

  const [open, setOpenItems] = useState<ReadonlySet<SettingsSection>>(
    () => new Set(),
  );
  const setOpen = useCallback((section: SettingsSection, next: boolean) => {
    setOpenItems((items) => {
      if (items.has(section) === next) return items;
      const copy = new Set(items);
      if (next) copy.add(section);
      else copy.delete(section);
      return copy;
    });
  }, []);
  const reveal = useCallback(
    (section: SettingsSection) => {
      setOpen(section, true);
      // O foco vai já ao botão do item, que contém o título. A rolagem leva o
      // título ao topo (o `scroll-margin` dele o deixa fora de baixo do
      // cabeçalho fixo) quando o conteúdo já abriu: antes, a página pode não
      // ter altura para isso.
      const title = document.getElementById(sectionItemId(section));
      title
        ?.closest<HTMLButtonElement>("button")
        ?.focus({ preventScroll: true });
      const content = title?.closest("li")?.querySelector("[data-group]");
      const until = performance.now() + 1000;
      const scroll = () => {
        if (content?.checkVisibility() || performance.now() > until) {
          title?.scrollIntoView({ block: "start" });
        } else {
          requestAnimationFrame(scroll);
        }
      };
      requestAnimationFrame(scroll);
    },
    [setOpen],
  );
  // O endereço com `#maps` abre Mapas (a troca de endereço volta para Geral).
  useEffect(() => {
    const section = sectionOfHash(window.location.hash);
    if (section) reveal(section);
  }, [reveal]);
  const items = useMemo(
    () => ({ isOpen: (s: SettingsSection) => open.has(s), setOpen, reveal }),
    [open, setOpen, reveal],
  );

  return (
    <TournamentSettingsContext value={value}>
      <SettingsItemsContext value={items}>{children}</SettingsItemsContext>
    </TournamentSettingsContext>
  );
}
