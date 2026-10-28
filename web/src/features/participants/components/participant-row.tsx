import { useTranslations } from "next-intl";

import type { components } from "@/shared/api/schema";
import { cn } from "@/shared/lib/cn";
import { Avatar } from "@/shared/ui/avatar";
import { DepartureMark } from "./departure-mark";
import { ParticipantCheckInButton } from "./participant-check-in-button";
import { RemoveParticipantButton } from "./remove-participant-button";
import { WithdrawParticipantButton } from "./withdraw-participant-button";

type Props = {
  tenant: string;
  tournament: string;
  participant: components["schemas"]["Participant"];
  /** Mostra se fez check-in. */
  showCheckIn: boolean;
  /** Mostra o seed (depois de iniciar, quando os seeds são a ordem da chave). */
  showSeed: boolean;
  /** Organizador durante o check-in: marca e desmarca. */
  canCheckIn: boolean;
  canRemove: boolean;
  /** Organizador com o torneio em andamento: marca a desistência. */
  canWithdraw: boolean;
};

export function ParticipantRow({
  tenant,
  tournament,
  participant: p,
  showCheckIn,
  showSeed,
  canCheckIn,
  canRemove,
  canWithdraw,
}: Props) {
  const t = useTranslations("Participants");
  const withdrawable = canWithdraw && p.seed != null && !p.withdrawn;

  return (
    <li className="flex flex-wrap items-center gap-x-3 gap-y-1 py-2">
      {showSeed && p.seed != null && (
        <span className="w-8 text-right text-sm tabular-nums text-muted">
          {t("seed", { seed: p.seed })}
        </span>
      )}
      <Avatar src={p.avatar_url} name={p.nickname} />
      <span className="flex-1 font-medium">{p.nickname}</span>
      <DepartureMark withdrawn={p.withdrawn} left={p.left} />
      {showCheckIn && (
        <span
          className={cn(
            "text-xs",
            p.checked_in ? "text-green-700 dark:text-green-400" : "text-muted",
          )}
        >
          {p.checked_in ? t("checkIn.checkedIn") : t("checkIn.notCheckedIn")}
        </span>
      )}
      {(canCheckIn || canRemove || withdrawable) && (
        // À direita; vão para a linha de baixo só quando não cabem (celular, várias ações).
        <span className="ml-auto flex flex-wrap justify-end gap-3">
          {canCheckIn && (
            <ParticipantCheckInButton
              tenant={tenant}
              tournament={tournament}
              userId={p.user_id}
              checkedIn={p.checked_in}
              nickname={p.nickname}
            />
          )}
          {canRemove && (
            <RemoveParticipantButton
              tenant={tenant}
              tournament={tournament}
              userId={p.user_id}
              nickname={p.nickname}
            />
          )}
          {withdrawable && (
            <WithdrawParticipantButton
              tenant={tenant}
              tournament={tournament}
              userId={p.user_id}
              nickname={p.nickname}
            />
          )}
        </span>
      )}
    </li>
  );
}
