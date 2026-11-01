"use client";

import { useTranslations } from "next-intl";
import { useState } from "react";

import { useRouter } from "@/i18n/navigation";
import { api } from "@/shared/api/client";
import type { components } from "@/shared/api/schema";
import { useApiAction } from "@/shared/api/use-api-action";
import { focusIfLost } from "@/shared/lib/focus";
import { ErrorText } from "@/shared/ui/error-text";
import {
  type OverflowAction,
  OverflowActions,
} from "@/shared/ui/overflow-actions";
import { TextInput } from "@/shared/ui/text-input";
import { useMarkMine } from "../hooks/use-live-match";
import { gameParams, gameRowId, type MatchTarget } from "../lib/match-target";
import styles from "./game-organizer-menu.module.scss";

type Props = {
  target: MatchTarget;
  game: components["schemas"]["GameDetail"];
  /** É a última partida da série: a única que pode ser anulada. */
  last: boolean;
  /** Abre o formulário de outro resultado (ou da correção) no cartão. */
  onEdit: () => void;
};

/**
 * O ⋯ da organização numa partida enviada, com o que é raro: lançar outro
 * resultado (ou corrigir a confirmada), fora da contestada, que o tem à
 * vista; e anular a última sem confirmar para ser jogada de novo, com a
 * nota do histórico no Modal (cancelar a apaga). Some sem nenhum dos dois.
 */
export function GameOrganizerMenu({ target, game, last, onEdit }: Props) {
  const t = useTranslations("Match.gameOrganizer");
  const router = useRouter();
  const markMine = useMarkMine();
  const { run, pending, error } = useApiAction();
  const [note, setNote] = useState("");
  const confirmed = game.status === "confirmed";
  const replay = () =>
    run(
      () =>
        api.POST(
          "/tenants/{slug}/tournaments/{tournament}/matches/{match}/games/{game}/replay",
          {
            params: gameParams(target, game.number),
            body: { comment: note.trim() || null },
          },
        ),
      () => {
        setNote("");
        markMine();
        router.refresh();
        focusIfLost(gameRowId(game.number));
      },
      // Tela para trás (a partida confirmou pelo prazo): mostra a de agora.
      () => router.refresh(),
    );

  const items: OverflowAction[] = [];
  if (game.status !== "contested")
    items.push({
      label: confirmed ? t("correct") : t("other"),
      onSelect: onEdit,
    });
  if (last && !confirmed)
    items.push({
      label: t("replay"),
      destructive: true,
      onSelect: replay,
      confirm: {
        title: t("replaySubmit", { number: game.number }),
        question: t("replayQuestion", { number: game.number }),
        extra: (
          <TextInput
            labelText={t("note")}
            value={note}
            onChange={(e) => setNote(e.target.value)}
            maxLength={280}
          />
        ),
        onCancel: () => setNote(""),
      },
    });
  if (items.length === 0) return null;

  return (
    <>
      <OverflowActions
        items={items}
        size="sm"
        pending={pending}
        label={t("more", { number: game.number })}
      />
      {error && (
        <div className={styles.error}>
          <ErrorText>{error}</ErrorText>
        </div>
      )}
    </>
  );
}
