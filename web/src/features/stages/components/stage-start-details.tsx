import { useTranslations } from "next-intl";

import { useTeamName } from "@/features/teams/hooks/use-team-name";
import type { components } from "@/shared/api/schema";
import { ListItem, OrderedList } from "@/shared/ui/carbon/list";
import { Stack } from "@/shared/ui/carbon/stack";
import { candidateState, entering, substitutes } from "../lib/candidates";

type Props = {
  entry: components["schemas"]["StageEntry"];
  /** A fase é a disputa de posições: a lista sai em pares (1 x 2, 3 x 4…). */
  placement: boolean;
};

/**
 * O que o Modal de "Iniciar fase N" mostra para conferir: quem entra, na
 * ordem da fase (na disputa de posições, os pares da primeira rodada), os
 * suplentes que ficam de fora e quem fica fora por não ter feito o check-in.
 */
export function StageStartDetails({ entry, placement }: Props) {
  const t = useTranslations("Stages");
  const teamName = useTeamName();
  const teams = entering(entry).map(({ team }) => ({
    id: team.id,
    name: teamName(team),
  }));
  const out = substitutes(entry).map((candidate) => teamName(candidate.team));
  const absent = entry.candidates
    .filter((candidate) => candidateState(candidate, entry) === "absent")
    .map((candidate) => teamName(candidate.team));

  const pairs: { id: number; text: string }[] = [];
  for (let i = 0; i < teams.length; i += 2) {
    const [a, b] = [teams[i], teams[i + 1]];
    pairs.push({
      id: a.id,
      text: b
        ? t("versus", { a: a.name, b: b.name })
        : t("alone", { name: a.name }),
    });
  }
  const lines = placement
    ? pairs
    : teams.map(({ id, name }) => ({ id, text: name }));

  return (
    <Stack gap={4}>
      <p>{t(placement ? "startPlacement" : "startEntering")}</p>
      <OrderedList>
        {lines.map(({ id, text }) => (
          <ListItem key={id}>{text}</ListItem>
        ))}
      </OrderedList>
      {out.length > 0 && (
        <p>{t("startSubstitutes", { names: out.join(", ") })}</p>
      )}
      {absent.length > 0 && (
        <p>{t("startAbsent", { names: absent.join(", ") })}</p>
      )}
    </Stack>
  );
}
