import { useTranslations } from "next-intl";

import type { components } from "@/shared/api/schema";
import { IconIndicator } from "@/shared/ui/icon-indicator";
import { StatusTag } from "@/shared/ui/status-tag";

type Props = {
  participant: components["schemas"]["Participant"];
  /** O check-in conta (aberto, ou já houve): mostra se fez ou não. */
  showCheckIn: boolean;
  /** Já começou: quem não fez o check-in ficou fora da chave. */
  started: boolean;
};

/**
 * O estado de um inscrito na tabela, onde se confere quem fez o quê. O caso
 * esperado não tem marca: "check-in feito" é só texto, e sem check-in valendo
 * a célula fica vazia. A exceção tem: o triângulo de quem ainda não fez e a
 * etiqueta, na cor de cancelado, de quem saiu, desistiu ou ficou fora da chave.
 */
export function ParticipantState({
  participant: p,
  showCheckIn,
  started,
}: Props) {
  const t = useTranslations("Participants.state");
  const cancelled = (label: string) => (
    <StatusTag status="cancelled">{label}</StatusTag>
  );
  if (p.left) return cancelled(t("left"));
  if (p.withdrawn) return cancelled(t("withdrawn"));
  if (!showCheckIn) return null;
  if (p.checked_in) return t("checkedIn");
  if (started) return cancelled(t("missedCheckIn"));
  return <IconIndicator kind="caution-minor" label={t("notCheckedIn")} />;
}
