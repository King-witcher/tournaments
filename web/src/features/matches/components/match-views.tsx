"use client";

import { useTranslations } from "next-intl";
import { type ReactNode, useDeferredValue, useId, useState } from "react";

import { Stack } from "@/shared/ui/carbon/stack";
import { ContentSwitcher } from "@/shared/ui/content-switcher";
import styles from "./match-views.module.scss";

type View = "rounds" | "crosstable";

type Props = {
  /** As séries por rodada, o padrão. */
  rounds: ReactNode;
  /** A tabela cruzada. */
  crosstable: ReactNode;
};

/**
 * As duas vistas das séries do round robin, no `ContentSwitcher`: por rodada
 * ou quem enfrentou quem. As duas chegam prontas do servidor. A vista nova
 * entra depois do clique (`useDeferredValue`): com muitos inscritos, montar
 * a tabela cruzada ou todas as rodadas segurava o switcher.
 */
export function MatchViews({ rounds, crosstable }: Props) {
  const t = useTranslations("Bracket.views");
  const [view, setView] = useState<View>("rounds");
  const shown = useDeferredValue(view);
  const id = useId();
  const tabId = (name: View) => `${id}-${name}`;

  return (
    <Stack gap={5} className={styles.views}>
      <div className={styles.switcher}>
        <ContentSwitcher
          aria-label={t("label")}
          options={[
            { value: "rounds", text: t("rounds"), id: tabId("rounds") },
            {
              value: "crosstable",
              text: t("crosstable"),
              id: tabId("crosstable"),
            },
          ]}
          value={view}
          onChange={setView}
        />
      </div>
      <div
        role="tabpanel"
        aria-labelledby={tabId(shown)}
        className={styles.panel}
      >
        {shown === "rounds" ? rounds : crosstable}
      </div>
    </Stack>
  );
}
