import { useTranslations } from "next-intl";

import type { components } from "@/shared/api/schema";
import { type Status, StatusTag } from "@/shared/ui/status-tag";

type Props = {
  participant: components["schemas"]["Participant"];
  /** O check-in conta (aberto, ou já houve): mostra se fez ou não. */
  showCheckIn: boolean;
  /** Já começou: quem não fez o check-in ficou fora da chave. */
  started: boolean;
};

/**
 * O estado de um inscrito como etiqueta: saiu (da comunidade), desistiu,
 * check-in feito ou que falta (fora da chave, depois de iniciar) ou só
 * inscrito.
 */
export function ParticipantState({
  participant: p,
  showCheckIn,
  started,
}: Props) {
  const t = useTranslations("Participants.state");
  const [status, label]: [Status, string] = p.left
    ? ["cancelled", t("left")]
    : p.withdrawn
      ? ["cancelled", t("withdrawn")]
      : !showCheckIn
        ? ["registration", t("registered")]
        : p.checked_in
          ? ["confirmed", t("checkedIn")]
          : started
            ? ["cancelled", t("missedCheckIn")]
            : ["check_in", t("notCheckedIn")];
  return <StatusTag status={status}>{label}</StatusTag>;
}
