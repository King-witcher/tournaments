"use client";

import { ArrowLeft } from "@carbon/icons-react";
import {
  type ComponentType,
  type ReactNode,
  useLayoutEffect,
  useRef,
} from "react";

import { Link as IntlLink } from "@/i18n/navigation";
import { useAppShell } from "./app-shell-context";
import styles from "./app-side-nav.module.scss";
import { SideNav, SideNavItems, SideNavLink } from "./carbon/ui-shell";

type Props = {
  /** O que o menu navega, para o leitor de tela ("Menu do torneio"). */
  label: string;
  /**
   * A área de cima, de onde se entrou nesta (a comunidade, no menu do
   * torneio): o primeiro item, com a seta para a esquerda.
   */
  back?: { href: string; label: string };
  /**
   * O topo do menu, acima dos links: o tipo da área num rótulo pequeno e o
   * nome dela (`AppSideNavTitle`), como no IBM Cloud.
   */
  header?: ReactNode;
  /** `AppSideNavLink`, `SideNavMenu` e `SideNavDivider`. */
  children: ReactNode;
};

// O link do Carbon aceita `as`, mas o tipo pede um componente sem props obrigatórias.
const NavLink = IntlLink as unknown as ComponentType;

/**
 * O menu lateral do UI Shell do Carbon (`SideNav`), um por área, como no IBM
 * Cloud: entrar num item (o torneio, dentro da comunidade) troca o menu pelo
 * dele, com a volta no topo (`back`). Fixo à esquerda do `lg` para cima, com
 * o conteúdo e o rodapé recuados (`AppShell`); abaixo, some e abre por cima
 * da página pelo botão de menu do cabeçalho. Pode ir em qualquer lugar da
 * página (é fixo); use num `layout.tsx`, para não remontar a cada navegação.
 * No máximo dois níveis (link e `SideNavMenu` com itens), como pede o Carbon.
 *
 * ```tsx
 * <AppSideNav
 *   label={t("nav")}
 *   back={{ href: `/${tenant}`, label: tenantName }}
 *   header={<AppSideNavTitle kind={t("kind")} name={name} />}
 * >
 *   <AppSideNavLink href={base} match="exact">{t("overview")}</AppSideNavLink>
 * </AppSideNav>
 * ```
 */
export function AppSideNav({ label, back, header, children }: Props) {
  const { navOpen, setNavOpen } = useAppShell();
  const marker = useRef<HTMLDivElement>(null);
  // O recuo da casca procura a marca (`:has([data-app-side-nav])`). Ela já
  // vem do servidor (a página não pula) e sai enquanto o menu está guardado
  // escondido: o Next mantém o layout de uma área que se deixou (`Activity`),
  // e as limpezas dos efeitos rodam quando ele some.
  useLayoutEffect(() => {
    const node = marker.current;
    node?.setAttribute("data-app-side-nav", "");
    return () => node?.removeAttribute("data-app-side-nav");
  }, []);
  return (
    <div ref={marker} className={styles.marker} data-app-side-nav="">
      <SideNav
        aria-label={label}
        expanded={navOpen}
        onOverlayClick={() => setNavOpen(false)}
        onSideNavBlur={() => setNavOpen(false)}
        className={styles.nav}
      >
        {back && (
          <SideNavItems className={styles.back}>
            <SideNavLink
              as={NavLink}
              {...{ href: back.href }}
              renderIcon={ArrowLeft}
            >
              {back.label}
            </SideNavLink>
          </SideNavItems>
        )}
        {header && <div className={styles.header}>{header}</div>}
        <SideNavItems>{children}</SideNavItems>
      </SideNav>
    </div>
  );
}
