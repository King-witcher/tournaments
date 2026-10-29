"use client";

import { useTranslations } from "next-intl";
import { type RefObject, useState } from "react";

import { FormModal } from "@/shared/ui/form-modal";
import { Select } from "@/shared/ui/select";
import { useMovePlayer } from "../hooks/use-move-player";
import { useTeamName } from "../hooks/use-team-name";
import {
  canMove,
  type Destination,
  type Formation,
  type Spot,
} from "../lib/formation";

type Props = {
  tenant: string;
  tournament: string;
  formation: Formation;
  userId: number;
  /** O apelido como aparece (já com "Conta excluída"). */
  name: string;
  from: Spot;
  returnFocusTo?: RefObject<HTMLElement | null>;
  onClose: () => void;
  onDone: (formation: Formation) => void;
};

/** O valor do `<option>` de volta para o destino. */
function parse(value: string): Destination {
  return value === "new" || value === "reserve" ? value : Number(value);
}

/**
 * "Mover para…" de um jogador, a alternativa ao arrastar (celular e
 * teclado): os times com vaga (com quantos têm), um time novo com ele como
 * capitão e a reserva, tirando o lugar onde ele já está.
 */
export function MovePlayerModal({
  tenant,
  tournament,
  formation,
  userId,
  name,
  from,
  returnFocusTo,
  onClose,
  onDone,
}: Props) {
  const t = useTranslations("Team.move");
  const teamName = useTeamName();
  const { move, pending, error } = useMovePlayer(tenant, tournament);
  const [to, setTo] = useState("");
  const teams = formation.teams.filter((team) =>
    canMove(formation, from, team.id),
  );

  return (
    <FormModal
      title={t("title", { name })}
      submitLabel={t("submit")}
      pending={pending}
      submitDisabled={to === ""}
      error={error}
      returnFocusTo={returnFocusTo}
      onClose={onClose}
      onSubmit={() => move(userId, from, parse(to), onDone)}
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
