"use client";

import { useTranslations } from "next-intl";
import { type RefObject, useState } from "react";

import { shownNickname } from "@/features/members/lib/nickname";
import { api } from "@/shared/api/client";
import type { components } from "@/shared/api/schema";
import { useApiAction } from "@/shared/api/use-api-action";
import { FormModal } from "@/shared/ui/form-modal";
import { Select } from "@/shared/ui/select";
import { useTeamName } from "../hooks/use-team-name";

type Props = {
  tenant: string;
  tournament: string;
  team: components["schemas"]["Team"];
  returnFocusTo?: RefObject<HTMLElement | null>;
  onClose: () => void;
  onDone: () => void;
};

/**
 * "Trocar capitão", para a organização até o torneio encerrar: o capitão
 * age pelo time no veto, e precisa ser um jogador do time que continua na
 * comunidade.
 */
export function CaptainModal({
  tenant,
  tournament,
  team,
  returnFocusTo,
  onClose,
  onDone,
}: Props) {
  const t = useTranslations("Team.captain");
  const tMember = useTranslations("Member");
  const teamName = useTeamName();
  const { run, pending, error } = useApiAction();
  const [captain, setCaptain] = useState(String(team.captain));

  return (
    <FormModal
      title={t("title", { name: teamName(team) })}
      submitLabel={t("submit")}
      pending={pending}
      error={error}
      returnFocusTo={returnFocusTo}
      onClose={onClose}
      onSubmit={() =>
        run(
          () =>
            api.PATCH("/tenants/{slug}/tournaments/{tournament}/teams/{team}", {
              params: { path: { slug: tenant, tournament, team: team.id } },
              body: { captain: Number(captain) },
            }),
          onDone,
        )
      }
    >
      <p>{t("hint")}</p>
      <Select
        labelText={t("label")}
        value={captain}
        onChange={(event) => setCaptain(event.target.value)}
      >
        {team.members
          .filter((m) => !m.left)
          .map((m) => (
            <option key={m.user_id} value={m.user_id}>
              {shownNickname(m.nickname, tMember("deletedAccount"))}
            </option>
          ))}
      </Select>
    </FormModal>
  );
}
