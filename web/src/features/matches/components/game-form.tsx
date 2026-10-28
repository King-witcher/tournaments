"use client";

import { useTranslations } from "next-intl";
import { useActionState, useState } from "react";

import { useRouter } from "@/i18n/navigation";
import { callApi } from "@/shared/api/call";
import { api } from "@/shared/api/client";
import type { components } from "@/shared/api/schema";
import { useApiErrorMessage } from "@/shared/api/use-api-error-message";
import { submitAction } from "@/shared/lib/submit-action";
import { Alert } from "@/shared/ui/alert";
import { Button } from "@/shared/ui/button";
import { Field } from "@/shared/ui/field";
import { Input } from "@/shared/ui/input";
import {
  EMPTY_GAME,
  type GameValues,
  gameInput,
  gameParams,
  type MatchTarget,
} from "../lib/match-target";
import { MapSelect } from "./map-select";
import { WinnerChoice } from "./winner-choice";

type Props = {
  target: MatchTarget;
  /** Número do mapa na série. */
  number: number;
  /** `report`: jogador envia (fica pendente); `organizer`: lança, resolve ou corrige, já confirmado. */
  mode: "report" | "organizer";
  names: { a: string; b: string };
  maps: components["schemas"]["Map"][];
  /** Valores para começar (o mapa enviado, ao resolver ou corrigir). */
  initial?: GameValues;
  submitLabel: string;
  /** Depois de salvar (fecha o formulário de quem abriu). */
  onDone?: () => void;
};

/** Um mapa da série: quem venceu, o mapa jogado e os frags opcionais. */
export function GameForm(props: Props) {
  const { target, number, mode, names, maps, initial, submitLabel, onDone } =
    props;
  const t = useTranslations("Match.form");
  const tMatch = useTranslations("Match");
  const errorMessage = useApiErrorMessage();
  const router = useRouter();
  const [game, setGame] = useState<GameValues>(initial ?? EMPTY_GAME);
  const [comment, setComment] = useState("");
  // Mapa antigo, sem nome: o organizador pode manter assim.
  const mapOptional = mode === "organizer" && initial != null && !initial.map;

  const [error, submit, pending] = useActionState(async () => {
    const { winner } = game;
    if (!winner) return t("chooseWinner");
    if (!game.map && !mapOptional) return t("chooseTheMap");
    const body = gameInput({ ...game, winner });
    const params = gameParams(target, number);
    const result =
      mode === "report"
        ? await callApi(
            api.POST(
              "/tenants/{slug}/tournaments/{tournament}/matches/{match}/games/{game}/report",
              { params, body },
            ),
          )
        : await callApi(
            api.PUT(
              "/tenants/{slug}/tournaments/{tournament}/matches/{match}/games/{game}",
              {
                params,
                body: { game: body, comment: comment.trim() || null },
              },
            ),
          );
    if (!result.ok) return errorMessage(result.code, result.params);
    setGame(EMPTY_GAME);
    onDone?.();
    router.refresh();
    return undefined;
  }, undefined);

  const frags = (side: "a" | "b") => (
    <Input
      type="number"
      min={0}
      max={999}
      inputMode="numeric"
      aria-label={t("fragsOf", { name: names[side] })}
      placeholder={names[side]}
      value={game[side]}
      onChange={(e) => setGame({ ...game, [side]: e.target.value })}
      className="w-24"
    />
  );

  return (
    <form onSubmit={submitAction(submit)} className="flex flex-col gap-3">
      <WinnerChoice
        legend={t("winner", { number })}
        names={names}
        value={game.winner}
        onChange={(winner) => setGame({ ...game, winner })}
      />
      <Field label={t("map")}>
        <MapSelect
          maps={maps}
          value={game.map}
          onChange={(map) => setGame({ ...game, map })}
        />
      </Field>
      <fieldset className="flex flex-col gap-1">
        <legend className="text-sm font-medium">{t("frags")}</legend>
        <div className="flex items-center gap-2">
          {frags("a")}
          <span className="text-sm opacity-60">{tMatch("vs")}</span>
          {frags("b")}
        </div>
      </fieldset>
      {mode === "organizer" && (
        <Field label={t("comment")}>
          <Input
            value={comment}
            onChange={(e) => setComment(e.target.value)}
            maxLength={280}
          />
        </Field>
      )}
      {error && <Alert>{error}</Alert>}
      <div>
        <Button type="submit" disabled={pending}>
          {submitLabel}
        </Button>
      </div>
    </form>
  );
}
