import { useTranslations } from "next-intl";
import { type KeyboardEvent, useEffect, useRef } from "react";

import type { components } from "@/shared/api/schema";
import { Avatar } from "@/shared/ui/avatar";
import { Button } from "@/shared/ui/button";
import { Input } from "@/shared/ui/input";

type Props = {
  /** Seed que fica se a ordem for salva (1 = melhor). */
  position: number;
  /** Quantos estão na lista: a última posição possível. */
  total: number;
  participant: components["schemas"]["Participant"];
  /** Marca quem está sem seed (quando só alguns estão). */
  showNoSeed: boolean;
  disabled: boolean;
  /**
   * Muda a cada vez que ele é movido pelo campo com Enter: o foco volta para o
   * campo na posição nova.
   */
  focusToken?: number;
  /** Ausente no primeiro. */
  onUp?: () => void;
  /** Ausente no último. */
  onDown?: () => void;
  /** Leva para a posição `to` (1 a `total`); `keepFocus` com Enter. */
  onMoveTo: (to: number, keepFocus: boolean) => void;
};

/**
 * Um elegível do seeding. A posição é um campo: digitar o número e sair do
 * campo (ou Enter) leva direto para lá, sem subir um por um; as setas
 * continuam para o ajuste fino.
 */
export function SeedingRow({
  position,
  total,
  participant: p,
  showNoSeed,
  disabled,
  focusToken,
  onUp,
  onDown,
  onMoveTo,
}: Props) {
  const t = useTranslations("Participants.seeding");
  const input = useRef<HTMLInputElement>(null);

  // O campo é recriado na posição nova (key): devolve o foco a ele.
  useEffect(() => {
    if (focusToken !== undefined) input.current?.focus();
  }, [focusToken]);

  function commit(field: HTMLInputElement, keepFocus: boolean) {
    const to = Number(field.value);
    if (field.value.trim() && Number.isInteger(to) && to !== position) {
      onMoveTo(Math.min(Math.max(to, 1), total), keepFocus);
    } else {
      field.value = String(position);
    }
  }

  function onKeyDown(e: KeyboardEvent<HTMLInputElement>) {
    if (e.key === "Enter") {
      e.preventDefault();
      commit(e.currentTarget, true);
    }
  }

  return (
    <li className="flex flex-wrap items-center gap-x-3 gap-y-2 py-2">
      <Input
        // Recriado quando a posição muda, para mostrar a nova.
        key={position}
        ref={input}
        type="number"
        inputMode="numeric"
        min={1}
        max={total}
        defaultValue={position}
        disabled={disabled}
        onBlur={(e) => commit(e.currentTarget, false)}
        onKeyDown={onKeyDown}
        aria-label={t("position", { name: p.nickname, total })}
        className="w-16 text-right tabular-nums sm:min-h-8 sm:py-1"
      />
      <Avatar src={p.avatar_url} name={p.nickname} />
      <span className="min-w-0 flex-1 font-medium">{p.nickname}</span>
      {showNoSeed && p.seed == null && (
        <span className="text-xs text-muted">{t("noSeed")}</span>
      )}
      <span className="flex gap-2">
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
      </span>
    </li>
  );
}
