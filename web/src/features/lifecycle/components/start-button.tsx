import { getTranslations } from "next-intl/server";

import { getParticipants } from "@/features/participants/server";
import type { components } from "@/shared/api/schema";
import { TransitionButton } from "./transition-button";

/** Menor chave que a API aceita (`tournament.not_enough_eligible`). */
const MIN_ELIGIBLE = 2;

type Props = {
  tenant: string;
  tournament: components["schemas"]["Tournament"];
};

/**
 * "Iniciar torneio" com a conta na confirmação: no check-in, quantos entram
 * na chave e quantos ficam de fora sem check-in. Com menos de 2 elegíveis, o
 * botão fica desabilitado e o motivo aparece embaixo.
 */
export async function StartButton({ tenant, tournament }: Props) {
  const t = await getTranslations("Lifecycle");
  const checkIn = tournament.status === "check_in";
  const list = checkIn
    ? await getParticipants(tenant, tournament.slug)
    : undefined;
  const eligible = list
    ? list.filter((p) => p.checked_in).length
    : tournament.participants;
  const confirm = !checkIn
    ? t("startConfirmNoCheckIn", { count: eligible })
    : list
      ? t("startConfirmCount", { in: eligible, out: list.length - eligible })
      : t("startConfirm");
  // Sem a lista (API falhou), não bloqueia: a API confere ao iniciar.
  const short = (!checkIn || list) && eligible < MIN_ELIGIBLE;

  return (
    <div className="flex flex-col gap-1">
      <TransitionButton
        tenant={tenant}
        slug={tournament.slug}
        transition="start"
        label={t("start")}
        confirm={confirm}
        variant="default"
        disabled={short}
      />
      {short && (
        <p className="max-w-xs text-sm text-muted">
          {t(checkIn ? "notEnoughCheckIn" : "notEnough", {
            min: MIN_ELIGIBLE,
            count: eligible,
          })}
        </p>
      )}
    </div>
  );
}
