"use client";

import { useTranslations } from "next-intl";
import { type RefObject, useState } from "react";

import { FormModal } from "@/shared/ui/form-modal";
import { Select } from "@/shared/ui/select";
import { useTeamName } from "../hooks/use-team-name";
import {
  canMove,
  type Destination,
  type Formation,
  type Spot,
} from "../lib/formation";

type Props = {
  formation: Formation;
  /** O apelido como aparece (já com "Conta excluída"). */
  name: string;
  from: Spot;
  returnFocusTo?: RefObject<HTMLElement | null>;
  onClose: () => void;
  onMove: (to: Destination) => void;
};

/** O valor do `<option>` de volta para o destino. */
function parse(value: string): Destination {
  return value === "new" || value === "reserve" ? value : Number(value);
}

/**
 * "Mover para…" de um jogador, a alternativa ao arrastar (celular e
 * teclado): os times com vaga (com quantos têm), um time novo com ele como
 * capitão e a reserva, tirando o lugar onde ele já está. Muda só o rascunho
 * da formação.
 */
export function MovePlayerModal({
  formation,
  name,
  from,
  returnFocusTo,
  onClose,
  onMove,
}: Props) {
  const t = useTranslations("Team.move");
  const teamName = useTeamName();
  const [to, setTo] = useState("");
  const teams = formation.teams.filter((team) =>
    canMove(formation, from, team.id),
  );

  return (
    <FormModal
      title={t("title", { name })}
      submitLabel={t("submit")}
      submitDisabled={to === ""}
      returnFocusTo={returnFocusTo}
      onClose={onClose}
      onSubmit={() => onMove(parse(to))}
    >
      <Select
        labelText={t("label")}
        value={to}
        onChange={(event) => setTo(event.target.value)}
      >
        <option value="">{t("choose")}</option>
        {teams.map((team) => (
          <option key={team.id} value={team.id}>
            {t("team", {
              name: teamName(team),
              count: team.members.length,
              size: formation.team_size,
            })}
          </option>
        ))}
        {canMove(formation, from, "new") && (
          <option value="new">{t("newTeam", { name })}</option>
        )}
        {canMove(formation, from, "reserve") && (
          <option value="reserve">{t("reserve")}</option>
        )}
      </Select>
    </FormModal>
  );
}
