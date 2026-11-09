"use client";

import { type ReactNode, useDeferredValue, useId, useState } from "react";

import { Stack } from "@/shared/ui/carbon/stack";
import { ContentSwitcher } from "@/shared/ui/content-switcher";
import styles from "./match-views.module.scss";

type Props = {
  /** O que o seletor escolhe, para leitor de tela. */
  label: string;
  /** Cada vista, já desenhada no servidor. */
  views: { key: string; label: string; content: ReactNode }[];
  /** A vista aberta ao chegar; padrão, a primeira. */
  defaultIndex?: number;
};

/**
 * Vistas das séries de uma fase no `ContentSwitcher` (por rodada ou quem
 * enfrentou quem; chave principal ou repescagem), um nível abaixo das abas de
 * fase, como o Carbon pede. A vista nova entra depois do clique
 * (`useDeferredValue`): com muitos inscritos, montar a tabela cruzada ou todas
 * as rodadas segurava o seletor. A escolha fica no cliente e sobrevive ao
 * `router.refresh()` do tempo real.
 */
export function MatchViews({ label, views, defaultIndex = 0 }: Props) {
  const [view, setView] = useState(views[defaultIndex].key);
  const shown = useDeferredValue(view);
  const id = useId();
  const tabId = (key: string) => `${id}-${key}`;

  return (
    <Stack gap={5} className={styles.views}>
      <div className={styles.switcher}>
        <ContentSwitcher
          aria-label={label}
          options={views.map((v) => ({
            value: v.key,
            text: v.label,
            id: tabId(v.key),
          }))}
          value={view}
          onChange={setView}
        />
      </div>
      <div
        role="tabpanel"
        aria-labelledby={tabId(shown)}
        className={styles.panel}
      >
        {views.find((v) => v.key === shown)?.content}
      </div>
    </Stack>
  );
}
