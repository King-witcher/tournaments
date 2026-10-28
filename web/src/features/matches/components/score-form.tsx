"use client";

import { useTranslations } from "next-intl";
import { useActionState, useState } from "react";

import { useRouter } from "@/i18n/navigation";
import { callApi } from "@/shared/api/call";
import { api } from "@/shared/api/client";
import type { components } from "@/shared/api/schema";
import { useApiErrorMessage } from "@/shared/api/use-api-error-message";
import { Alert } from "@/shared/ui/alert";
import { Button } from "@/shared/ui/button";
import { Field } from "@/shared/ui/field";
import { Input } from "@/shared/ui/input";
import { Select } from "@/shared/ui/select";
import {
  type GameValues,
  gameInput,
  type MatchTarget,
  matchParams,
} from "../lib/match-target";
import { GamesEditor } from "./games-editor";

type Props = {
  target: MatchTarget;
  /** `report`: jogador envia; `organizer`: lança ou corrige, já confirmado. */
  mode: "report" | "organizer";
  bestOf: number;
  names: { a: string; b: string };
  maps: components["schemas"]["Map"][];
  /** Placar para começar (o enviado, ao resolver ou corrigir). */
  initial?: { a: number; b: number };
  submitLabel: string;
};

/** Placar da série em mapas e o detalhe opcional por mapa. */
export function ScoreForm(props: Props) {
  const { target, mode, bestOf, names, maps, initial, submitLabel } = props;
  const t = useTranslations("Match");
  const errorMessage = useApiErrorMessage();
  const router = useRouter();
  const needed = Math.floor(bestOf / 2) + 1;
  const [wins, setWins] = useState(initial ?? { a: 0, b: 0 });
  const [games, setGames] = useState<GameValues[]>([]);
  const [comment, setComment] = useState("");

  const [error, submit, pending] = useActionState(async () => {
    if (games.some((g) => !g.map)) return t("report.incomplete");
    const score = {
      a_wins: wins.a,
      b_wins: wins.b,
      games: games.map(gameInput),
    };
    const params = matchParams(target);
    const result =
      mode === "report"
        ? await callApi(
            api.POST(
              "/tenants/{slug}/tournaments/{tournament}/matches/{match}/report",
              { params, body: score },
            ),
          )
        : await callApi(
            api.PUT(
              "/tenants/{slug}/tournaments/{tournament}/matches/{match}/result",
              {
                params,
                body: {
                  result: { kind: "played", ...score },
                  comment: comment.trim() || null,
                },
              },
            ),
          );
    if (!result.ok) return errorMessage(result.code, result.params);
    router.refresh();
    return undefined;
  }, undefined);

  const winsField = (side: "a" | "b") => (
    <Field label={t("report.wins", { name: names[side] })}>
      <Select
        value={wins[side]}
        onChange={(e) => setWins({ ...wins, [side]: Number(e.target.value) })}
        className="w-24"
      >
        {Array.from({ length: needed + 1 }, (_, n) => n).map((n) => (
          <option key={n} value={n}>
            {n}
          </option>
        ))}
      </Select>
    </Field>
  );

  return (
    <form action={submit} className="flex flex-col gap-4">
      <div className="flex flex-wrap gap-4">
        {winsField("a")}
        {winsField("b")}
      </div>
      <GamesEditor
        games={games}
        onChange={setGames}
        maps={maps}
        names={names}
        max={wins.a + wins.b > 0 ? wins.a + wins.b : bestOf}
      />
      {mode === "organizer" && (
        <Field label={t("organizer.comment")}>
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
