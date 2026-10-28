"use client";

import { useTranslations } from "next-intl";
import { useState } from "react";

import { useRouter } from "@/i18n/navigation";
import { api } from "@/shared/api/client";
import type { components } from "@/shared/api/schema";
import { useApiAction } from "@/shared/api/use-api-action";
import { ErrorText } from "@/shared/ui/alert";
import { Button } from "@/shared/ui/button";
import { ConfirmButton } from "@/shared/ui/confirm-button";
import { SeedingRow } from "./seeding-row";

type Participant = components["schemas"]["Participant"];

type Props = {
  tenant: string;
  tournament: string;
  /** Os elegíveis na ordem que vale agora. */
  initial: Participant[];
};

/**
 * Ordem do seeding: digitar a posição (vai direto), subir e descer, salvar a
 * ordem ou sortear.
 */
export function SeedingEditor({ tenant, tournament, initial }: Props) {
  const t = useTranslations("Participants.seeding");
  const router = useRouter();
  const { run, pending, error } = useApiAction();
  const [order, setOrder] = useState(initial);
  const [focus, setFocus] = useState<{ id: number; n: number }>();
  const params = { path: { slug: tenant, tournament } };
  const changed = order.some((p, i) => p.user_id !== initial[i].user_id);
  // Ninguém tem seed: iniciar assim sorteia, e a ordem da tela não vale.
  const noneSeeded = initial.every((p) => p.seed == null);
  // Ainda há quem não tem seed: salvar grava a ordem que aparece.
  const unsaved = changed || initial.some((p) => p.seed == null);

  function move(index: number, delta: -1 | 1) {
    setOrder((current) => {
      const next = [...current];
      [next[index], next[index + delta]] = [next[index + delta], next[index]];
      return next;
    });
  }

  /** Pelo id: um blur atrasado de um campo já movido não mexe em outro. */
  function moveTo(userId: number, to: number, keepFocus: boolean) {
    setOrder((current) => {
      const from = current.findIndex((p) => p.user_id === userId);
      if (from < 0 || from === to - 1) return current;
      const next = [...current];
      const [moved] = next.splice(from, 1);
      next.splice(to - 1, 0, moved);
      return next;
    });
    if (keepFocus) setFocus((f) => ({ id: userId, n: (f?.n ?? 0) + 1 }));
  }

  const shuffle = () =>
    run(
      () =>
        api.POST("/tenants/{slug}/tournaments/{tournament}/seeding/shuffle", {
          params,
        }),
      () => router.refresh(),
    );

  return (
    <div className="flex flex-col gap-3">
      {noneSeeded && <p className="text-sm">{t("noneSeeded")}</p>}
      <ol className="divide-y divide-black/10 dark:divide-white/10">
        {order.map((p, i) => (
          <SeedingRow
            key={p.user_id}
            position={i + 1}
            total={order.length}
            participant={p}
            showNoSeed={!noneSeeded}
            disabled={pending}
            onUp={i > 0 ? () => move(i, -1) : undefined}
            onDown={i < order.length - 1 ? () => move(i, 1) : undefined}
            onMoveTo={(to, keepFocus) => moveTo(p.user_id, to, keepFocus)}
            focusToken={focus?.id === p.user_id ? focus.n : undefined}
          />
        ))}
      </ol>
      {changed && <p className="text-sm">{t("changed")}</p>}
      <div className="flex flex-wrap gap-3">
        <Button
          size="sm"
          pending={pending}
          disabled={!unsaved}
          onClick={() =>
            run(
              () =>
                api.PUT("/tenants/{slug}/tournaments/{tournament}/seeding", {
                  params,
                  body: { user_ids: order.map((p) => p.user_id) },
                }),
              () => router.refresh(),
            )
          }
        >
          {t("save")}
        </Button>
        {changed ? (
          <ConfirmButton
            pending={pending}
            danger={false}
            question={t("shuffleConfirm")}
            onConfirm={shuffle}
          >
            {t("shuffle")}
          </ConfirmButton>
        ) : (
          <Button
            size="sm"
            variant="outline"
            pending={pending}
            onClick={shuffle}
          >
            {t("shuffle")}
          </Button>
        )}
      </div>
      {error && <ErrorText>{error}</ErrorText>}
    </div>
  );
}
