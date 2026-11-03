"use client";

import { useTranslations } from "next-intl";

import { shownNickname } from "@/features/members/lib/nickname";
import { vetoResultText } from "../lib/veto-result";
import { useVetoBoard } from "./use-veto-board";

/** O resultado do veto à vista, em texto para copiar; vazio sem nada ainda. */
export function useVetoResult(): string {
  const tGames = useTranslations("Match.games");
  const tMember = useTranslations("Member");
  const { veto, names } = useVetoBoard();
  if (!veto) return "";
  return vetoResultText(veto, {
    names,
    game: (number) => tGames("game", { number }),
    nickname: (nickname) => shownNickname(nickname, tMember("deletedAccount")),
  });
}
