"use client";

import { useTranslations } from "next-intl";
import {
  type ComponentProps,
  useEffect,
  useRef,
  useSyncExternalStore,
} from "react";

import { PaginationNav as CarbonPaginationNav } from "./carbon/pagination-nav";
import { usePrefix } from "./carbon/prefix";

type Props = Omit<
  ComponentProps<typeof CarbonPaginationNav>,
  "translateWithId" | "ref"
>;

// Abaixo do md (672px): 5 páginas de 48px e as setas cabem em 390px. O
// Carbon só reduz abaixo de 320px.
const PHONE = "(max-width: 41.98rem)";
const PHONE_ITEMS = 5;

function subscribePhone(onChange: () => void) {
  const query = window.matchMedia(PHONE);
  query.addEventListener("change", onChange);
  return () => query.removeEventListener("change", onChange);
}

/**
 * Números de página com as setas (`PaginationNav` do Carbon), para passar
 * entre páginas de conteúdo (as rodadas de um torneio, as partidas de uma
 * série) perto do que elas mostram. Lista de dados com "por página" é a
 * `Pagination`. `page` começa em 0; `totalItems` é o número de páginas.
 * Todos os textos do Carbon saem traduzidos, inclusive o do "…" das páginas
 * escondidas, que o Carbon deixa em inglês. No celular, mostra no máximo 5
 * páginas de uma vez, para não rolar para o lado.
 */
export function PaginationNav({
  size = "md",
  itemsShown = 10,
  ...rest
}: Props) {
  const t = useTranslations("Common");
  const prefix = usePrefix();
  const nav = useRef<HTMLElement>(null);
  const phone = useSyncExternalStore(
    subscribePhone,
    () => window.matchMedia(PHONE).matches,
    () => false,
  );
  const page = t("paginationNav.page");
  const active = `${t("paginationNav.active")}, ${page}`;
  const selectPage = t("paginationNav.selectPage");

  const texts: Record<string, string> = {
    "carbon.pagination-nav.next": t("table.nextPage"),
    "carbon.pagination-nav.previous": t("table.previousPage"),
    "carbon.pagination-nav.item": page,
    "carbon.pagination-nav.active": t("paginationNav.active"),
    "carbon.pagination-nav.of": t("paginationNav.of"),
  };

  // O Carbon não passa o `translateWithId` ao "…": o select dele se chama
  // "Select Page number" e, quando esconde uma página só, o botão dela diz
  // "Page". Corrige no DOM sempre que a lista muda (o React não reescreve um
  // atributo ou texto cujo valor não mudou).
  useEffect(() => {
    const root = nav.current;
    if (!root) return;
    const fix = () => {
      for (const select of root.querySelectorAll("select")) {
        if (select.getAttribute("aria-label") !== selectPage)
          select.setAttribute("aria-label", selectPage);
      }
      const labels = root.querySelectorAll(
        `button .${prefix}--pagination-nav__accessibility-label`,
      );
      for (const label of labels) {
        if (label.textContent !== page && label.textContent !== active)
          label.textContent = page;
      }
    };
    fix();
    const observer = new MutationObserver(fix);
    observer.observe(root, { childList: true, subtree: true });
    return () => observer.disconnect();
  }, [prefix, page, active, selectPage]);

  return (
    <CarbonPaginationNav
      ref={nav}
      size={size}
      itemsShown={phone ? Math.min(itemsShown, PHONE_ITEMS) : itemsShown}
      translateWithId={(id) => texts[id]}
      {...rest}
    />
  );
}
