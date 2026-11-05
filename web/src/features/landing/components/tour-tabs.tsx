"use client";

import type { ReactNode } from "react";

import { Tab, TabPanel, TabPanels, Tabs } from "@/shared/ui/carbon/tabs";
import { TabList } from "@/shared/ui/tab-list";
import styles from "./tour-tabs.module.scss";

type Props = {
  /** O que as abas são, para leitor de tela. */
  label: string;
  /** Cada parte do site, já desenhada no servidor. */
  tabs: { key: string; label: string; content: ReactNode }[];
};

/**
 * As partes do site em abas, uma captura por vez, como a segunda faixa do
 * tutorial do Carbon: a página não desce as capturas uma embaixo da outra.
 */
export function TourTabs({ label, tabs }: Props) {
  return (
    <Tabs>
      <TabList aria-label={label}>
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
