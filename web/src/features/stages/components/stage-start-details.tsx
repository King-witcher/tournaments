import { useTranslations } from "next-intl";

import { useTeamName } from "@/features/teams/hooks/use-team-name";
import type { components } from "@/shared/api/schema";
import { Stack } from "@/shared/ui/carbon/stack";
import { candidateState, entering, substitutes } from "../lib/candidates";

type Props = {
  entry: components["schemas"]["StageEntry"];
  /** O torneio é por times: "Entram 8 times". */
  tdm: boolean;
};

/**
 * O que o Modal de "Iniciar fase N" mostra para conferir: quantos entram e as
 * exceções, quem fica de fora (suplentes e sem check-in). A ordem e os pares
 * se veem na chave logo depois, e o início se desfaz enquanto nada foi jogado.
 * Lista longa: o Modal do Carbon rola o corpo, com os botões à vista.
 */
export function StageStartDetails({ entry, tdm }: Props) {
  const t = useTranslations("Stages");
  const teamName = useTeamName();
  const out = substitutes(entry).map((candidate) => teamName(candidate.team));
  const absent = entry.candidates
    .filter((candidate) => candidateState(candidate, entry) === "absent")
    .map((candidate) => teamName(candidate.team));

  return (
    <Stack gap={4}>
      <p>
        {t("enters", {
          count: entering(entry).length,
          tdm: tdm ? "yes" : "no",
        })}
      </p>
      {out.length > 0 && (
        <p>{t("startSubstitutes", { names: out.join(", ") })}</p>
      )}
      {absent.length > 0 && (
        <p>{t("startAbsent", { names: absent.join(", ") })}</p>
      )}
    </Stack>
  );
}
