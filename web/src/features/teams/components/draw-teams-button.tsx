"use client";

import { useTranslations } from "next-intl";
import { type RefObject, useRef, useState } from "react";

import { api } from "@/shared/api/client";
import { useApiAction } from "@/shared/api/use-api-action";
import { focusIfLost } from "@/shared/lib/focus";
import { ActionWithNote } from "@/shared/ui/action-with-note";
import { Button } from "@/shared/ui/button";
import { FormModal } from "@/shared/ui/form-modal";
import { Notice } from "@/shared/ui/notice";
import { RadioTileGroup } from "@/shared/ui/radio-tile-group";
import { TEAMS_ID } from "../lib/focus";
import { drawPool, type Formation } from "../lib/formation";

type Props = {
  tenant: string;
  tournament: string;
  formation: Formation;
  /** Há mudança não salva: o Modal avisa que ela se perde. */
  unsaved: boolean;
  /** Sorteou: a formação nova. */
  onDrawn: (formation: Formation) => void;
};

/**
 * "Sortear times", ao lado do título dos times: abre o `DrawTeamsModal`. Sem
 * gente para um time, desabilitado, com o motivo embaixo.
 */
export function DrawTeamsButton({
  tenant,
  tournament,
  formation,
  unsaved,
  onDrawn,
}: Props) {
  const t = useTranslations("Team.draw");
  const trigger = useRef<HTMLButtonElement>(null);
  const [open, setOpen] = useState(false);
  const pool = drawPool(formation);
  const size = formation.team_size;
  const blocked =
    pool >= size
      ? undefined
      : t(formation.has_check_in ? "shortCheckIn" : "short", {
          size,
          count: pool,
        });

  return (
    <ActionWithNote align="end" note={blocked}>
      <Button
        ref={trigger}
        kind="tertiary"
        size="sm"
        disabled={blocked !== undefined}
        onClick={() => setOpen(true)}
      >
        {t("open")}
      </Button>
      {open && (
        <DrawTeamsModal
          tenant={tenant}
          tournament={tournament}
          formation={formation}
          unsaved={unsaved}
          trigger={trigger}
          onClose={() => setOpen(false)}
          onDrawn={(data) => {
            setOpen(false);
            onDrawn(data);
            focusIfLost(TEAMS_ID);
          }}
        />
      )}
    </ActionWithNote>
  );
}

/**
 * O Modal aberto: a escolha entre balanceado e aleatório e o que o resultado
 * não mostra de antemão (o que se desfaz, o que não foi salvo). Um a cada
 * abertura, para voltar ao padrão.
 */
function DrawTeamsModal({
  tenant,
  tournament,
  formation,
  unsaved,
  trigger,
  onClose,
  onDrawn,
}: Omit<Props, "onDrawn"> & {
  trigger: RefObject<HTMLButtonElement | null>;
  onClose: () => void;
  onDrawn: (formation: Formation) => void;
}) {
  const t = useTranslations("Team.draw");
  const { run, pending, error } = useApiAction();
  const [method, setMethod] = useState<"balanced" | "random">("balanced");
  const pool = drawPool(formation);
  const left = pool % formation.team_size;
  const hasOrder = formation.teams.some((team) => team.seed != null);

  return (
    <FormModal
      title={t("title")}
      submitLabel={t("submit")}
      pending={pending}
      error={error}
      returnFocusTo={trigger}
      onClose={onClose}
      onSubmit={() =>
        run(
          () =>
            api.POST("/tenants/{slug}/tournaments/{tournament}/teams/draw", {
              params: { path: { slug: tenant, tournament } },
              body: { method },
            }),
          onDrawn,
        )
      }
    >
      <RadioTileGroup
        legend={t("method")}
        value={method}
        onChange={setMethod}
        options={[
          {
            value: "balanced",
            content: t("balanced"),
            description: t("balancedHint"),
          },
          {
            value: "random",
            content: t("random"),
            description: t("randomHint"),
          },
        ]}
      />
      <p>
        {t(formation.has_check_in ? "poolCheckIn" : "pool", { count: pool })}
        {left > 0 && ` ${t("leftover", { count: left })}`}
      </p>
      {formation.teams.length > 0 && (
        <Notice kind="warning">
          {t(hasOrder ? "replaceTeamsAndOrder" : "replaceTeams")}
        </Notice>
      )}
      {unsaved && <Notice kind="warning">{t("discardDraft")}</Notice>}
    </FormModal>
  );
}
