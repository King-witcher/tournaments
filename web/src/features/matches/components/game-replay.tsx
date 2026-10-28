"use client";

import { useTranslations } from "next-intl";
import { useState } from "react";

import { useRouter } from "@/i18n/navigation";
import { api } from "@/shared/api/client";
import { useApiAction } from "@/shared/api/use-api-action";
import { focusIfLost } from "@/shared/lib/focus";
import { ErrorText } from "@/shared/ui/alert";
import { Button } from "@/shared/ui/button";
import { Field } from "@/shared/ui/field";
import { Input } from "@/shared/ui/input";
import { gameParams, gameRowId, type MatchTarget } from "../lib/match-target";

type Props = { target: MatchTarget; number: number };

/**
 * Para a organização, no último mapa ainda sem confirmar: anular para os
 * jogadores jogarem de novo (queda, revanche combinada). O primeiro clique
 * abre a explicação e a nota; só o segundo anula, como no `ConfirmButton`.
 */
export function GameReplay({ target, number }: Props) {
  const t = useTranslations("Match.gameOrganizer");
  const router = useRouter();
  const { run, pending, error } = useApiAction();
  const [open, setOpen] = useState(false);
  const [comment, setComment] = useState("");

  const replay = () =>
    run(
      () =>
        api.POST(
          "/tenants/{slug}/tournaments/{tournament}/matches/{match}/games/{game}/replay",
          {
            params: gameParams(target, number),
            body: { comment: comment.trim() || null },
          },
        ),
      () => {
        router.refresh();
        // A linha do mapa anulado vira a do próximo envio, com o mesmo número.
        focusIfLost(gameRowId(number));
      },
      // Tela para trás (o mapa já confirmou pelo prazo): mostra o estado de agora.
      () => router.refresh(),
    );

  if (!open) {
    return (
      <Button
        size="sm"
        variant="outline"
        disabled={pending}
        onClick={() => setOpen(true)}
      >
        {t("replay")}
      </Button>
    );
  }
  return (
    // basis-full: abre numa linha própria, abaixo dos outros botões do mapa.
    <fieldset
      className="flex min-w-0 basis-full flex-col gap-2"
      onKeyDown={(event) => {
        if (event.key === "Escape") setOpen(false);
      }}
    >
      <legend className="mb-2 max-w-prose text-sm">
        {t("replayQuestion", { number })}
      </legend>
      <Field label={t("note")}>
        <Input
          value={comment}
          onChange={(e) => setComment(e.target.value)}
          maxLength={280}
        />
      </Field>
      <div className="flex flex-wrap gap-2">
        <Button
          size="sm"
          variant="destructive"
          pending={pending}
          onClick={replay}
        >
          {t("replaySubmit", { number })}
        </Button>
        <Button size="sm" variant="outline" onClick={() => setOpen(false)}>
          {t("cancel")}
        </Button>
      </div>
      {error && <ErrorText>{error}</ErrorText>}
    </fieldset>
  );
}
