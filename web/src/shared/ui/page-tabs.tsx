"use client";

import { Tab, TabList, TabPanel, TabPanels, Tabs } from "@carbon/react";
import type { ComponentType, ReactNode } from "react";

import { Link as IntlLink, usePathname } from "@/i18n/navigation";
import styles from "./page-tabs.module.scss";

type PageTab = {
  /** Sub-rota da aba (sem o idioma), ex.: `/${tenant}/members`. */
  href: string;
  label: string;
};

type Props = {
  tabs: PageTab[];
  /** O que as abas são, para leitor de tela ("Seções da comunidade"). */
  label: string;
  /** O conteúdo da aba atual (o `children` do `layout.tsx` das sub-rotas). */
  children?: ReactNode;
};

/** A aba do endereço atual: a de href igual ou, senão, a de prefixo mais longo. */
function currentIndex(tabs: PageTab[], pathname: string) {
  const exact = tabs.findIndex((tab) => tab.href === pathname);
  if (exact !== -1) return exact;
  let best = -1;
  tabs.forEach((tab, index) => {
    if (
      pathname.startsWith(`${tab.href}/`) &&
      (best === -1 || tab.href.length > tabs[best].href.length)
    )
      best = index;
  });
  return Math.max(best, 0);
}

// O Tab aceita `as`, mas o tipo pede um componente sem props obrigatórias.
const TabLink = IntlLink as unknown as ComponentType;

/**
 * Abas do Carbon que são links para sub-rotas (uma coisa por aba, cada uma
 * com o próprio `loading.tsx`); a atual sai do endereço. Use no `layout.tsx`
 * do segmento, com as sub-rotas em `children`, que vão no painel da aba.
 * Setas movem entre as abas; Enter abre.
 */
export function PageTabs({ tabs, label, children }: Props) {
  const pathname = usePathname();
  const selected = currentIndex(tabs, pathname);
  return (
    <Tabs selectedIndex={selected}>
      <TabList aria-label={label} activation="manual">
        {tabs.map((tab) => (
          <Tab key={tab.href} as={TabLink} {...{ href: tab.href }}>
            {tab.label}
          </Tab>
        ))}
      </TabList>
      {children !== undefined && (
        <TabPanels>
          {tabs.map((tab, index) => (
            <TabPanel key={tab.href} className={styles.panel}>
              {index === selected ? children : null}
            </TabPanel>
          ))}
        </TabPanels>
      )}
    </Tabs>
  );
}
