"use client";

import { useTranslations } from "next-intl";

import { Button } from "@/shared/ui/button";
import { Stack } from "@/shared/ui/carbon/stack";
import { Notice } from "@/shared/ui/notice";
import { SortableList } from "@/shared/ui/sortable-list";
import { TextInput } from "@/shared/ui/text-input";
import type { Tie, TieDraft, TieTeam } from "../lib/ties";
import styles from "./tie-form.module.scss";
import { TieTeamRow } from "./tie-team-row";

/** Até onde vai o motivo (a API recusa acima disso). */
const MAX_REASON = 280;

export type TieStatus = "pending" | "saved" | "gone";

type Props = {
  tie: Tie;
  status: TieStatus;
  /** A ordem e o motivo escolhidos; salvo, os que foram gravados. */
  draft: TieDraft;
  /** É o último pendente: salvar encerra o torneio ou a fase. */
  ending?: "tournament" | "stage";
  onChange: (draft: TieDraft) => void;
};

/**
 * Um empate no Modal: os empatados, com os números da classificação, para
 * arrastar até a ordem final (abre na da chave, que é o que vale se ninguém
 * mexer; a linha acima da lista diz qual ordem está valendo e, mexida, oferece
 * voltar), o motivo opcional e o aviso de que a ordem salva não muda e, no
 * último pendente, do que termina. Salvo aqui, a mesma lista só para leitura;
 * decidido por outra pessoa, só o aviso.
 */
export function TieForm({ tie, status, draft, ending, onChange }: Props) {
  const t = useTranslations("Bracket.ties");
  const seedOrder = tie.teams.map((team) => team.id);
  const moved = draft.order.join() !== seedOrder.join();
  const teams = draft.order
    .map((id) => tie.teams.find((team) => team.id === id))
    .filter((team): team is TieTeam => team !== undefined);

  if (status === "gone") return <Notice kind="info">{t("gone")}</Notice>;

  const list = (
    <SortableList
      items={teams}
      itemLabel={(team) => team.name}
      disabled={status === "saved"}
      renderItem={(team, index) => (
        <TieTeamRow position={tie.position + index} team={team} />
      )}
      onChange={(items) =>
        onChange({ ...draft, order: items.map((team) => team.id) })
      }
    />
  );

  if (status === "saved") {
    return (
      <Stack gap={5}>
        <Notice kind="success">{t("saved")}</Notice>
        {list}
        {draft.reason.trim() && (
          <p className={styles.reason}>
            {t("savedReason", { reason: draft.reason.trim() })}
          </p>
        )}
      </Stack>
    );
  }

  return (
    <Stack gap={5}>
      <div>
        <div className={styles.orderBar}>
          <span className={styles.orderLabel}>
            {t(moved ? "orderChanged" : "orderSeed")}
          </span>
          {moved && (
            <Button
              kind="ghost"
              size="sm"
              onClick={() => onChange({ ...draft, order: seedOrder })}
            >
              {t("resetSeed")}
            </Button>
          )}
        </div>
        {list}
      </div>
      <TextInput
        labelText={t("reason")}
        helperText={t("reasonHint")}
        value={draft.reason}
        maxLength={MAX_REASON}
        onChange={(e) => onChange({ ...draft, reason: e.target.value })}
      />
      {ending ? (
        <Notice kind="warning">
          {`${t("final")} ${t(ending === "tournament" ? "ends" : "endsStage")}`}
        </Notice>
      ) : (
        <p className={styles.note}>{t("final")}</p>
      )}
    </Stack>
  );
}
