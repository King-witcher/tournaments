import { useTranslations } from "next-intl";

import type { components } from "@/shared/api/schema";
import { Stack } from "@/shared/ui/carbon/stack";
import { Section } from "@/shared/ui/section";
import { isEvenSeries } from "../lib/format";
import { countingTiebreakers, stageTiebreakers } from "../lib/tiebreakers";
import { AutoRulesAbsence } from "./auto-rules-absence";
import { AutoRulesEntry } from "./auto-rules-entry";
import { AutoRulesFormat } from "./auto-rules-format";
import { AutoRulesMaps } from "./auto-rules-maps";
import { AutoRulesResults } from "./auto-rules-results";
import { AutoRulesSeries } from "./auto-rules-series";
import { AutoRulesTable } from "./auto-rules-table";
import { AutoRulesTeams } from "./auto-rules-teams";

type Props = {
  tournament: components["schemas"]["Tournament"];
  /** Os mapas ativos, para o nome de cada mapa do pool. */
  maps: components["schemas"]["Map"][];
  /** Quem vê ainda edita o torneio: a seção diz de onde o texto sai. */
  editable?: boolean;
};

/**
 * As regras que saem da configuração do torneio, para quem joga: o que
 * precisa saber e fazer, na ordem em que acontece, em trechos com título
 * curto. Inscrição e check-in (só antes de começar), os times (TDM), a
 * chave, a classificação (todos contra todos e suíço, com os desempates da
 * fase), cada série, os mapas do pool, o resultado e o W.O. Cada trecho segue
 * a configuração e muda sozinho quando ela muda, porque nada é gravado. O que ela não diz
 * (conduta, queda de conexão, quem cria a sala) fica nas regras adicionais.
 */
export function TournamentAutoRules({ tournament, maps, editable }: Props) {
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
        {(stage.format === "round_robin" || stage.format === "swiss") && (
          <AutoRulesTable
            format={stage.format}
            even={isEvenSeries(stage.best_of)}
            tiebreakers={countingTiebreakers(
              stageTiebreakers(stage) ?? [],
              stage.format,
              stage.best_of,
            )}
            finalTie={stage.final_tie ?? "seed"}
          />
        )}
        <AutoRulesSeries tournament={tournament} />
        <AutoRulesMaps tournament={tournament} maps={maps} />
        <AutoRulesResults tournament={tournament} />
        <AutoRulesAbsence tournament={tournament} />
      </Stack>
    </Section>
  );
}
