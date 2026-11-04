"use client";

import { Stack } from "@carbon/react";
import { useTranslations } from "next-intl";
import { type ReactNode, useId, useState } from "react";

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
 * ou quem enfrentou quem. As duas chegam prontas do servidor.
 */
export function MatchViews({ rounds, crosstable }: Props) {
  const t = useTranslations("Bracket.views");
  const [view, setView] = useState<View>("rounds");
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
        aria-labelledby={tabId(view)}
        className={styles.panel}
      >
        {view === "rounds" ? rounds : crosstable}
      </div>
    </Stack>
  );
}
