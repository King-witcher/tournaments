import { Stack } from "@carbon/react";
import { useTranslations } from "next-intl";

import type { components } from "@/shared/api/schema";
import { Section } from "@/shared/ui/section";
import { AutoRulesAbsence } from "./auto-rules-absence";
import { AutoRulesEntry } from "./auto-rules-entry";
import { AutoRulesFormat } from "./auto-rules-format";
import { AutoRulesResults } from "./auto-rules-results";
import { AutoRulesSeries } from "./auto-rules-series";
import { AutoRulesTable } from "./auto-rules-table";
import { AutoRulesTeams } from "./auto-rules-teams";

type Props = {
  tournament: components["schemas"]["Tournament"];
  /** Quem vê ainda edita o torneio: a seção diz de onde o texto sai. */
  editable?: boolean;
};

/**
 * As regras que saem da configuração do torneio, para quem joga: o que
 * precisa saber e fazer, na ordem em que acontece, em trechos com título
 * curto. Inscrição e check-in (só antes de começar), os times (TDM), a
 * chave, a classificação (todos contra todos e suíço), cada série, o
 * resultado e o W.O. Cada trecho segue a configuração e muda sozinho quando
 * ela muda, porque nada é gravado. O que ela não diz (conduta, queda de
 * conexão, quem cria a sala) fica nas regras adicionais.
 */
export function TournamentAutoRules({ tournament, editable }: Props) {
  const t = useTranslations("Tournament.autoRules");
  const [stage] = tournament.stages;
  const started =
    tournament.status === "in_progress" || tournament.status === "finished";

  return (
    <Section
      title={t("title")}
      description={editable ? t("generated") : undefined}
    >
      <Stack gap={7}>
        {!started && <AutoRulesEntry tournament={tournament} />}
        {tournament.mode === "tdm" && (
          <AutoRulesTeams tournament={tournament} />
        )}
        <AutoRulesFormat tournament={tournament} />
        {stage.format !== "elimination" && (
          <AutoRulesTable format={stage.format} />
        )}
        <AutoRulesSeries tournament={tournament} />
        <AutoRulesResults tournament={tournament} />
        <AutoRulesAbsence tournament={tournament} />
      </Stack>
    </Section>
  );
}
