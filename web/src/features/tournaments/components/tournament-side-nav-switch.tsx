"use client";

import type { ReactNode } from "react";

import { usePathname } from "@/i18n/navigation";
import { AppSideNav } from "@/shared/ui/app-side-nav";
import { AppSideNavLink } from "@/shared/ui/app-side-nav-link";
import { SideNavDivider, SideNavMenu } from "@/shared/ui/carbon/ui-shell";
import type { SettingsSection } from "../lib/settings-sections";
import { settingsHref } from "../lib/tabs";
import { type NavTab, useTournamentNav } from "./tournament-nav-context";

type Item = { slug: string; header: ReactNode; tabs: NavTab[] };

type Props = {
  tenant: string;
  /** O nome do menu, para o leitor de tela ("Menu do torneio"). */
  label: string;
  /** A volta à comunidade. */
  back: { href: string; label: string };
  items: Item[];
  /** As seções das Configurações, na ordem, com o nome de cada uma. */
  settingsSections: { section: SettingsSection; label: string }[];
  /** O menu da comunidade, fora de um torneio. */
  children: ReactNode;
};

const MANAGE = /\/manage$/;
const SETTINGS = /\/settings$/;

/**
 * Troca o menu da comunidade pelo do torneio quando o endereço é de um (a
 * seção dele, a partida, a ordem da chave, os times). As seções são as da
 * página, quando ela já disse quais são; senão, as da lista. Organização e
 * Configurações, só da organização, ficam num grupo à parte; Configurações
 * se desdobra nas seções dela (o segundo nível do menu), aberta quando o
 * endereço é de uma.
 */
export function TournamentSideNavSwitch({
  tenant,
  label,
  back,
  items,
  settingsSections,
  children,
}: Props) {
  const pathname = usePathname();
  const { current } = useTournamentNav();
  const item = items.find((x) => x.slug === pathname.split("/")[2]);
  // A navegação do cliente para `/` (o início, que o proxy reescreve) não
  // desmonta o layout da comunidade: fora dela, nenhum menu.
  if (pathname.split("/")[1] !== tenant) return null;
  if (!item) return children;

  const base = `/${tenant}/${item.slug}`;
  const tabs = current?.slug === item.slug ? current.tabs : item.tabs;
  const link = (tab: NavTab) => (
    <AppSideNavLink
      key={tab.href}
      href={tab.href}
      // A Visão geral é o endereço do torneio, prefixo das outras.
      match={tab.href === base ? "exact" : "prefix"}
    >
      {tab.label}
    </AppSideNavLink>
  );
  const main = tabs.filter(
    (tab) => !MANAGE.test(tab.href) && !SETTINGS.test(tab.href),
  );
  const manage = tabs.find((tab) => MANAGE.test(tab.href));
  const settings = tabs.find((tab) => SETTINGS.test(tab.href));
  const inSettings =
    settings !== undefined &&
    (pathname === settings.href || pathname.startsWith(`${settings.href}/`));

  return (
    <AppSideNav label={label} back={back} header={item.header}>
      {main.map(link)}
      {(manage || settings) && <SideNavDivider />}
      {manage && link(manage)}
      {settings && (
        <SideNavMenu
          // Outro torneio, outro menu: abre de novo conforme o endereço.
          key={item.slug}
          title={settings.label}
          defaultExpanded={inSettings}
        >
          {settingsSections.map(({ section, label: sectionLabel }) => {
            const href = settingsHref(base, section);
            return (
              <AppSideNavLink
                key={section}
                href={href}
                // Geral é o endereço das Configurações, prefixo das outras.
                match={section === "general" ? "exact" : "prefix"}
                inMenu
              >
                {sectionLabel}
              </AppSideNavLink>
            );
          })}
        </SideNavMenu>
      )}
    </AppSideNav>
  );
}
