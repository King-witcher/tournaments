"use client";

import { ArrowDown, ArrowUp } from "@carbon/icons-react";
import { useTranslations } from "next-intl";
import {
  type KeyboardEvent,
  type RefObject,
  useEffect,
  useRef,
  useState,
} from "react";

import { shownNickname } from "@/features/members/lib/nickname";
import type { components } from "@/shared/api/schema";
import { Avatar } from "@/shared/ui/avatar";
import { IconButton } from "@/shared/ui/icon-button";
import { NumberInput } from "@/shared/ui/number-input";
import { StatusTag } from "@/shared/ui/status-tag";
import { useTeamName } from "../hooks/use-team-name";
import { soloMember } from "../lib/team-name";
import styles from "./seeding-row.module.scss";

type Props = {
  /** Seed que fica se a ordem for salva (1 = melhor). */
  position: number;
  /** Quantos estão na lista: a última posição possível. */
  total: number;
  team: components["schemas"]["Team"];
  /** Marca quem está sem seed (quando só alguns estão). */
  showNoSeed: boolean;
  /** O rating na comunidade ("Rating 1569", "Sem rating"); sem nada quando ninguém da lista tem. */
  ratingText?: string;
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
 * Um time elegível do seeding (no duelo, o jogador). A posição é um campo: digitar o número e sair do
 * campo (ou Enter) leva direto para lá, sem subir um por um; as setas
 * continuam para o ajuste fino.
 */
export function SeedingRow({
  position,
  total,
  team,
  showNoSeed,
  ratingText,
  disabled,
  focusToken,
  onUp,
  onDown,
  onMoveTo,
}: Props) {
  const t = useTranslations("Participants.seeding");
  const tMember = useTranslations("Member");
  const name = useTeamName()(team);
  const solo = soloMember(team);
  const input = useRef<HTMLInputElement>(null);

  // O campo é recriado na posição nova (key): devolve o foco a ele.
  useEffect(() => {
    if (focusToken !== undefined) input.current?.focus();
  }, [focusToken]);

  return (
    <li className={styles.row}>
      <PositionField
        // Recriado quando a posição muda, para mostrar a nova.
        key={position}
        inputRef={input}
        position={position}
        total={total}
        label={t("position", { name, total })}
        disabled={disabled}
        onMoveTo={onMoveTo}
      />
      <span className={styles.name}>
        <Avatar src={solo?.avatar_url} name={name} />
        <span className={styles.label}>
          {name}
          {/* No TDM, quem está no time, para reconhecer o "Time de {capitão}". */}
          {!solo && (
            <span className={styles.members}>
              {team.members
                .map((m) =>
                  shownNickname(m.nickname, tMember("deletedAccount")),
                )
                .join(", ")}
            </span>
          )}
        </span>
      </span>
      {ratingText && <span className={styles.rating}>{ratingText}</span>}
      {showNoSeed && team.seed == null && (
        <StatusTag status="neutral" size="sm">
          {t("noSeed")}
        </StatusTag>
      )}
      <span className={styles.moves}>
        <IconButton
          size="sm"
          label={t("up", { name })}
          disabled={disabled || !onUp}
          onClick={onUp}
        >
          <ArrowUp />
        </IconButton>
        <IconButton
          size="sm"
          label={t("down", { name })}
          disabled={disabled || !onDown}
          onClick={onDown}
        >
          <ArrowDown />
        </IconButton>
      </span>
    </li>
  );
}

/** O campo da posição, com o número que está sendo digitado. */
function PositionField({
  inputRef,
  position,
  total,
  label,
  disabled,
  onMoveTo,
}: {
  inputRef: RefObject<HTMLInputElement | null>;
  position: number;
  total: number;
  label: string;
  disabled: boolean;
  onMoveTo: (to: number, keepFocus: boolean) => void;
}) {
  const [value, setValue] = useState<number | null>(position);

  function commit(keepFocus: boolean) {
    if (value !== null && Number.isInteger(value) && value !== position) {
      onMoveTo(Math.min(Math.max(value, 1), total), keepFocus);
    } else {
      setValue(position);
    }
  }

  function onKeyDown(event: KeyboardEvent<HTMLInputElement>) {
    if (event.key === "Enter") {
      // Enter move, não envia o formulário.
      event.preventDefault();
      commit(true);
    }
  }

  return (
    <div className={styles.position}>
      <NumberInput
        ref={inputRef}
        labelText={label}
        hideLabel
        hideSteppers
        allowEmpty
        disableWheel
        size="sm"
        inputMode="numeric"
        value={value}
        onChange={setValue}
        disabled={disabled}
        onBlur={() => commit(false)}
        onKeyDown={onKeyDown}
      />
    </div>
  );
}
