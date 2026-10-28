"use client";

import { Tab, TabList, TabPanel, TabPanels, Tabs } from "@carbon/react";
import type { ReactNode } from "react";

import styles from "./bracket-tabs.module.scss";

type Props = {
  /** O que as abas são, para leitor de tela. */
  label: string;
  /** Cada parte da chave, já desenhada no servidor. */
  tabs: { key: string; label: string; content: ReactNode }[];
  /** A aba aberta ao chegar (a da partida que espera quem vê). */
  defaultIndex: number;
};

/**
 * As partes da dupla em abas (a principal com a grande final, e a
 * repescagem), para a página não descer uma chave embaixo da outra. As
 * abas `contained` se distinguem das do torneio, logo acima. A escolha fica
 * no cliente e sobrevive ao `router.refresh()` do tempo real.
 */
export function BracketTabs({ label, tabs, defaultIndex }: Props) {
  return (
    <Tabs defaultSelectedIndex={defaultIndex}>
      <TabList aria-label={label} contained>
        {tabs.map((tab) => (
          <Tab key={tab.key}>{tab.label}</Tab>
        ))}
      </TabList>
      <TabPanels>
        {tabs.map((tab) => (
          <TabPanel key={tab.key} className={styles.panel}>
            {tab.content}
          </TabPanel>
        ))}
      </TabPanels>
    </Tabs>
  );
}
