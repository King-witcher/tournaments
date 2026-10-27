import type { components } from "@/shared/api/schema";
import { Avatar } from "@/shared/ui/avatar";
import { RemoveParticipantButton } from "./remove-participant-button";

type Props = {
  tenant: string;
  tournament: string;
  participant: components["schemas"]["Participant"];
  canRemove: boolean;
};

export function ParticipantRow({
  tenant,
  tournament,
  participant,
  canRemove,
}: Props) {
  return (
    <li className="flex items-center gap-3 py-2">
      <Avatar src={participant.avatar_url} />
      <span className="flex-1 font-medium">{participant.nickname}</span>
      {canRemove && (
        <RemoveParticipantButton
          tenant={tenant}
          tournament={tournament}
          userId={participant.user_id}
        />
      )}
    </li>
  );
}
