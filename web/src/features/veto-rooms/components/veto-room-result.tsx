"use client";

import { useTranslations } from "next-intl";

import { CopyVetoResult } from "@/features/matches/components/copy-veto-result";
import { DraftSummary } from "@/features/matches/components/draft-summary";
import { VetoMapBoard } from "@/features/matches/components/veto-map-board";
import { useVetoBoard } from "@/features/matches/hooks/use-veto-board";
import { useVetoResult } from "@/features/matches/hooks/use-veto-result";
import { Column, Grid } from "@/shared/ui/carbon/grid";
import { CodeSnippet } from "@/shared/ui/code-snippet";
import { Section } from "@/shared/ui/section";

/**
 * O fim da sala: o texto para o chat, à vista e com o botão de copiar, e como
 * ficou (o resumo do draft, com os mapas; sem draft, os mapas).
 */
export function VetoRoomResult() {
  const t = useTranslations("VetoRoom");
  const { veto } = useVetoBoard();
  const result = useVetoResult();
  if (!veto) return null;
  return (
    <Section title={t("result")}>
      <CodeSnippet
        type="multi"
        wrapText
        hideCopyButton
        aria-label={t("resultText")}
      >
        {result}
      </CodeSnippet>
      <CopyVetoResult kind="primary" size="md" />
      {veto.champion_phase ? (
        <Grid>
          <Column sm={4} md={8} lg={8}>
            <DraftSummary />
          </Column>
        </Grid>
      ) : (
        <VetoMapBoard />
      )}
    </Section>
  );
}
