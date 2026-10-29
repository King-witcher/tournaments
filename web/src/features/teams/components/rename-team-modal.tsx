"use client";

import { useTranslations } from "next-intl";
import { type RefObject, useState } from "react";

import { api } from "@/shared/api/client";
import type { components } from "@/shared/api/schema";
import { useApiAction } from "@/shared/api/use-api-action";
import { FormModal } from "@/shared/ui/form-modal";
import { TextInput } from "@/shared/ui/text-input";
import { useTeamName } from "../hooks/use-team-name";
import type { Formation } from "../lib/formation";

/** O mesmo limite da API (`team.name_invalid {max}`). */
const MAX_NAME = 40;

type Props = {
  tenant: string;
  tournament: string;
  team: components["schemas"]["Team"];
  /** O botão que abriu, para o foco voltar a ele. */
  returnFocusTo?: RefObject<HTMLElement | null>;
  onClose: () => void;
  /** Gravou: a formação que a API devolveu. */
  onDone: (formation: Formation) => void;
};

/**
 * O Modal de renomear um time (a organização e o capitão, até o torneio
 * encerrar): o nome atual no campo; vazio, o time volta a aparecer como
 * "Time de {capitão}". O erro do nome (longo, repetido) vai no campo; o
 * resto, acima dos botões.
 */
export function RenameTeamModal({
  tenant,
  tournament,
  team,
  returnFocusTo,
  onClose,
  onDone,
}: Props) {
  const t = useTranslations("Team.rename");
  const teamName = useTeamName();
  const { run, pending, error } = useApiAction();
  const [name, setName] = useState(team.name ?? "");
  const [onField, setOnField] = useState(false);

  return (
    <FormModal
      title={t("title", { name: teamName(team) })}
      submitLabel={t("submit")}
      pending={pending}
      error={onField ? undefined : error}
      returnFocusTo={returnFocusTo}
      onClose={onClose}
      onSubmit={() =>
        run(
          () =>
            api.PATCH("/tenants/{slug}/tournaments/{tournament}/teams/{team}", {
              params: { path: { slug: tenant, tournament, team: team.id } },
              body: { name: name.trim() === "" ? null : name },
            }),
          onDone,
          (code) => setOnField(code.startsWith("team.name_")),
        )
      }
    >
      <TextInput
        labelText={t("label")}
        value={name}
        maxLength={MAX_NAME}
        onChange={(event) => setName(event.target.value)}
        helperText={t("helper", {
          fallback: teamName({ ...team, name: null }),
        })}
        invalid={onField && error !== undefined}
        invalidText={error}
      />
    </FormModal>
  );
}
