"use client";

import { useTranslations } from "next-intl";

import type { components } from "@/shared/api/schema";
import { SortableList } from "@/shared/ui/sortable-list";
import { useTeamName } from "../hooks/use-team-name";
import { SeedingRow } from "./seeding-row";

type Team = components["schemas"]["Team"];

type Props = {
  order: Team[];
  /** Sem números quando a ordem ainda não vale (ninguém tem seed). */
  numbered: boolean;
  /** Marca quem está sem seed (quando só alguns estão). */
  showNoSeed?: boolean;
  /** O texto do rating de cada time; sem ele, a coluna não aparece. */
  ratingText?: (team: Team) => string;
  /** A ordem nova. Sem ele, só consulta: sem as alças. */
  onChange?: (order: Team[]) => void;
};

const noChange = () => {};

/**
 * A ordem da chave: os times em posição, arrastando pela alça (mouse, toque e
 * teclado) ou, sem `onChange`, só para ler, como o registro do que definiu
 * a chave depois de iniciar.
 */
export function SeedingList({
  order,
  numbered,
  showNoSeed = false,
  ratingText,
  onChange,
}: Props) {
  const t = useTranslations("Participants.seeding");
  const teamName = useTeamName();
  return (
    <SortableList
      label={t("listLabel")}
      items={order}
      itemLabel={teamName}
      disabled={onChange === undefined}
      onChange={onChange ?? noChange}
      renderItem={(team, index) => (
        <SeedingRow
          position={index + 1}
          numbered={numbered}
          team={team}
          showNoSeed={showNoSeed}
          ratingText={ratingText?.(team)}
        />
      )}
    />
  );
}
