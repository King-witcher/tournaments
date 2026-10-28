import { useTranslations } from "next-intl";

import type { components } from "@/shared/api/schema";
import { cn } from "@/shared/lib/cn";
import { Avatar } from "@/shared/ui/avatar";
import { ParticipantCheckInButton } from "./participant-check-in-button";
import { RemoveParticipantButton } from "./remove-participant-button";

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
};

export function ParticipantRow({
  tenant,
  tournament,
  participant: p,
  showCheckIn,
  showSeed,
  canCheckIn,
  canRemove,
}: Props) {
  const t = useTranslations("Participants");

  return (
    <li className="flex items-center gap-3 py-2">
      {showSeed && p.seed != null && (
        <span className="w-8 text-right text-sm tabular-nums opacity-70">
          {t("seed", { seed: p.seed })}
        </span>
      )}
      <Avatar src={p.avatar_url} />
      <span className="flex-1 font-medium">{p.nickname}</span>
      {showCheckIn && (
        <span
          className={cn(
            "text-xs",
            p.checked_in ? "text-green-700 dark:text-green-400" : "opacity-60",
          )}
        >
          {p.checked_in ? t("checkIn.checkedIn") : t("checkIn.notCheckedIn")}
        </span>
      )}
      {canCheckIn && (
        <ParticipantCheckInButton
          tenant={tenant}
          tournament={tournament}
          userId={p.user_id}
          checkedIn={p.checked_in}
        />
      )}
      {canRemove && (
        <RemoveParticipantButton
          tenant={tenant}
          tournament={tournament}
          userId={p.user_id}
        />
      )}
    </li>
  );
}
