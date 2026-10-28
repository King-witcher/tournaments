"use client";

import { useTranslations } from "next-intl";
import { useState } from "react";

import { useRouter } from "@/i18n/navigation";
import { api } from "@/shared/api/client";
import type { components } from "@/shared/api/schema";
import { useApiAction } from "@/shared/api/use-api-action";
import { ErrorText } from "@/shared/ui/alert";
import { Button } from "@/shared/ui/button";
import { SeedingRow } from "./seeding-row";

type Participant = components["schemas"]["Participant"];

type Props = {
  tenant: string;
  tournament: string;
  /** Os elegíveis na ordem que vale agora. */
  initial: Participant[];
};

/** Ordem do seeding: subir e descer, salvar a ordem ou sortear. */
export function SeedingEditor({ tenant, tournament, initial }: Props) {
  const t = useTranslations("Participants.seeding");
  const router = useRouter();
  const { run, pending, error } = useApiAction();
  const [order, setOrder] = useState(initial);
  const params = { path: { slug: tenant, tournament } };
  const changed = order.some((p, i) => p.user_id !== initial[i].user_id);
  // Ainda há quem não tem seed: salvar grava a ordem que aparece.
  const unsaved = changed || initial.some((p) => p.seed == null);

  function move(index: number, delta: -1 | 1) {
    setOrder((current) => {
      const next = [...current];
      [next[index], next[index + delta]] = [next[index + delta], next[index]];
      return next;
    });
  }

  return (
    <div className="flex flex-col gap-3">
      <ol className="divide-y divide-black/10 dark:divide-white/10">
        {order.map((p, i) => (
          <SeedingRow
            key={p.user_id}
            position={i + 1}
            participant={p}
            disabled={pending}
            onUp={i > 0 ? () => move(i, -1) : undefined}
            onDown={i < order.length - 1 ? () => move(i, 1) : undefined}
          />
        ))}
      </ol>
      {changed && <p className="text-sm">{t("changed")}</p>}
      <div className="flex flex-wrap gap-3">
        <Button
          size="sm"
          disabled={pending || !unsaved}
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
        <Button
          size="sm"
          variant="outline"
          disabled={pending}
          onClick={() => {
            if (changed && !window.confirm(t("shuffleConfirm"))) return;
            run(
              () =>
                api.POST(
                  "/tenants/{slug}/tournaments/{tournament}/seeding/shuffle",
                  { params },
                ),
              () => router.refresh(),
            );
          }}
        >
          {t("shuffle")}
        </Button>
      </div>
      {error && <ErrorText>{error}</ErrorText>}
    </div>
  );
}
