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

/** Quem pode entrar no time. */
export type Candidate = { userId: number; nickname: string };

type Props = {
  tenant: string;
  tournament: string;
  team: components["schemas"]["Team"];
  /** Os reservas que continuam na comunidade, na ordem de inscrição. */
  reserves: Candidate[];
  /** Membros ativos da comunidade que não estão inscritos. */
  others: Candidate[];
  returnFocusTo?: RefObject<HTMLElement | null>;
  onClose: () => void;
  onDone: () => void;
};

/**
 * "Substituir jogador", para a organização com o torneio em andamento: quem
 * sai (um jogador do time, mesmo quem saiu da comunidade) e quem entra (um
 * reserva ou outro membro ativo, em grupos separados). Vale a partir do
 * próximo mapa enviado; quem sai fica inscrito, de reserva.
 */
export function SubstitutionModal({
  tenant,
  tournament,
  team,
  reserves,
  others,
  returnFocusTo,
  onClose,
  onDone,
}: Props) {
  const t = useTranslations("Team.substitute");
  const tMember = useTranslations("Member");
  const teamName = useTeamName();
  const { run, pending, error } = useApiAction();
  const [out, setOut] = useState("");
  const [enter, setEnter] = useState("");
  const none = reserves.length === 0 && others.length === 0;
  const option = (c: Candidate) => (
    <option key={c.userId} value={c.userId}>
      {shownNickname(c.nickname, tMember("deletedAccount"))}
    </option>
  );

  return (
    <FormModal
      title={t("title", { name: teamName(team) })}
      submitLabel={t("submit")}
      pending={pending}
      submitDisabled={none || out === "" || enter === ""}
      error={error}
      returnFocusTo={returnFocusTo}
      onClose={onClose}
      onSubmit={() =>
        run(
          () =>
            api.POST(
              "/tenants/{slug}/tournaments/{tournament}/teams/{team}/substitution",
              {
                params: { path: { slug: tenant, tournament, team: team.id } },
                body: { out: Number(out), in: Number(enter) },
              },
            ),
          onDone,
        )
      }
    >
      <p>{t("hint")}</p>
      {none ? (
        <p>{t("noCandidates")}</p>
      ) : (
        <>
          <Select
            labelText={t("out")}
            value={out}
            onChange={(event) => setOut(event.target.value)}
          >
            <option value="">{t("choose")}</option>
            {team.members.map((m) => {
              const name = shownNickname(m.nickname, tMember("deletedAccount"));
              return (
                <option key={m.user_id} value={m.user_id}>
                  {m.left ? t("left", { name }) : name}
                </option>
              );
            })}
          </Select>
          <Select
            labelText={t("in")}
            value={enter}
            onChange={(event) => setEnter(event.target.value)}
          >
            <option value="">{t("choose")}</option>
            {reserves.length > 0 && (
              <optgroup label={t("reserves")}>{reserves.map(option)}</optgroup>
            )}
            {others.length > 0 && (
              <optgroup label={t("members")}>{others.map(option)}</optgroup>
            )}
          </Select>
        </>
      )}
    </FormModal>
  );
}
