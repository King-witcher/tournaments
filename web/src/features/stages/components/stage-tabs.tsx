"use client";

import { type ReactNode, useOptimistic, useTransition } from "react";

import { usePathname, useRouter } from "@/i18n/navigation";
import { Tab, TabPanel, TabPanels, Tabs } from "@/shared/ui/carbon/tabs";
import { TabList } from "@/shared/ui/tab-list";
import styles from "./stage-tabs.module.scss";

type Props = {
  /** O que as abas são, para leitor de tela. */
  label: string;
  /** `value` é o que vai em `?stage=` (o número da fase ou `final`). */
  options: { value: string; text: string }[];
  /** A opção que a página mostra agora. */
  value: string;
  /** O conteúdo da opção de agora, já desenhado no servidor. */
  children: ReactNode;
};

/**
 * As abas de fase da Chave e da Classificação (conteúdos diferentes, por isso
 * abas e não `ContentSwitcher`; o Carbon põe o `ContentSwitcher` um nível
 * abaixo, dentro da aba). A escolha vai no endereço (`?stage=2`): sobrevive
 * ao `router.refresh()` do tempo real e dá link. Ativação manual: cada aba
 * busca a fase no servidor, então a seta só move o foco e Enter abre. A aba
 * marca na hora, antes de a página nova chegar.
 */
export function StageTabs({ label, options, value, children }: Props) {
  const router = useRouter();
  const pathname = usePathname();
  const [shown, setShown] = useOptimistic(value);
  const [, startTransition] = useTransition();
  const selected = Math.max(
    0,
    options.findIndex((option) => option.value === shown),
  );

  return (
    <Tabs
      selectedIndex={selected}
      onChange={({ selectedIndex }) =>
        startTransition(() => {
          const next = options[selectedIndex].value;
          setShown(next);
          router.replace(
            { pathname, query: { stage: next } },
            { scroll: false },
          );
        })
      }
    >
      <TabList aria-label={label} activation="manual">
        {options.map((option) => (
          <Tab key={option.value}>{option.text}</Tab>
        ))}
      </TabList>
      <TabPanels>
        {options.map((option, index) => (
          <TabPanel key={option.value} className={styles.panel}>
            {index === selected ? children : null}
          </TabPanel>
        ))}
      </TabPanels>
    </Tabs>
  );
}
