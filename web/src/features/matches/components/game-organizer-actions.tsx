"use client";

import { useTranslations } from "next-intl";
import { useState } from "react";

import { useRouter } from "@/i18n/navigation";
import { api } from "@/shared/api/client";
import type { components } from "@/shared/api/schema";
import { useApiAction } from "@/shared/api/use-api-action";
import { ErrorText } from "@/shared/ui/alert";
import { Button } from "@/shared/ui/button";
import { gameParams, type MatchTarget } from "../lib/match-target";
import { GameForm } from "./game-form";

type Schemas = components["schemas"];

type Props = {
  target: MatchTarget;
  game: Schemas["GameDetail"];
  names: { a: string; b: string };
  maps: Schemas["Map"][];
  /** Quem vê já tem o "confirmar" de jogador neste mapa. */
  responds: boolean;
};

/**
 * Para organizer e owner, num mapa enviado: confirmar agora (pendente),
 * aceitar como enviado (contestado) e lançar outro resultado no lugar
 * (resolver ou corrigir), já confirmado.
 */
export function GameOrganizerActions({
  target,
  game,
  names,
  maps,
  responds,
}: Props) {
  const t = useTranslations("Match.gameOrganizer");
  const tForm = useTranslations("Match.form");
  const router = useRouter();
  const { run, pending, error } = useApiAction();
  const [editing, setEditing] = useState(false);
  const confirmable =
    game.status === "contested" || (game.status === "pending" && !responds);

  const confirm = () =>
    run(
      () =>
        api.POST(
          "/tenants/{slug}/tournaments/{tournament}/matches/{match}/games/{game}/confirm",
          { params: gameParams(target, game.number) },
        ),
      () => router.refresh(),
    );

  if (editing) {
    return (
      <div className="flex flex-col gap-2">
        {game.status === "confirmed" && (
          <p className="text-xs opacity-70">{t("correctHint")}</p>
        )}
        <GameForm
          target={target}
          number={game.number}
          mode="organizer"
          names={names}
          maps={maps}
          initial={{
            winner: game.winner,
            map: game.map?.slug ?? "",
            a: game.a_frags?.toString() ?? "",
            b: game.b_frags?.toString() ?? "",
          }}
          submitLabel={tForm("save", { number: game.number })}
          onDone={() => setEditing(false)}
        />
        <div>
          <Button variant="link" onClick={() => setEditing(false)}>
            {t("cancel")}
          </Button>
        </div>
      </div>
    );
  }

  return (
    <div className="flex flex-col gap-1">
      <div className="flex flex-wrap gap-2">
        {confirmable && (
          <Button size="sm" disabled={pending} onClick={confirm}>
            {game.status === "contested" ? t("accept") : t("confirmNow")}
          </Button>
        )}
        <Button
          size="sm"
          variant="outline"
          disabled={pending}
          onClick={() => setEditing(true)}
        >
          {game.status === "confirmed" ? t("correct") : t("other")}
        </Button>
      </div>
      {error && <ErrorText>{error}</ErrorText>}
    </div>
  );
}
