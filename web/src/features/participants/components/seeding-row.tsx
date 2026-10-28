import { useTranslations } from "next-intl";

import type { components } from "@/shared/api/schema";
import { Avatar } from "@/shared/ui/avatar";
import { Button } from "@/shared/ui/button";

type Props = {
  /** Seed que fica se a ordem for salva (1 = melhor). */
  position: number;
  participant: components["schemas"]["Participant"];
  /** Marca quem está sem seed (quando só alguns estão). */
  showNoSeed: boolean;
  disabled: boolean;
  /** Ausente no primeiro. */
  onUp?: () => void;
  /** Ausente no último. */
  onDown?: () => void;
};

export function SeedingRow({
  position,
  participant: p,
  showNoSeed,
  disabled,
  onUp,
  onDown,
}: Props) {
  const t = useTranslations("Participants.seeding");

  return (
    <li className="flex items-center gap-3 py-2">
      <span className="w-6 text-right text-sm tabular-nums text-muted">
        {position}
      </span>
      <Avatar src={p.avatar_url} name={p.nickname} />
      <span className="flex-1 font-medium">{p.nickname}</span>
      {showNoSeed && p.seed == null && (
        <span className="text-xs text-muted">{t("noSeed")}</span>
      )}
      <Button
        variant="outline"
        size="sm"
        disabled={disabled || !onUp}
        onClick={onUp}
        aria-label={t("up", { name: p.nickname })}
      >
        ↑
      </Button>
      <Button
        variant="outline"
        size="sm"
        disabled={disabled || !onDown}
        onClick={onDown}
        aria-label={t("down", { name: p.nickname })}
      >
        ↓
      </Button>
    </li>
  );
}
