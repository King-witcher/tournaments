"use client";

import { Form, Stack } from "@carbon/react";
import { useTranslations } from "next-intl";
import { useState } from "react";

import { useRouter } from "@/i18n/navigation";
import { api } from "@/shared/api/client";
import type { components } from "@/shared/api/schema";
import { useApiAction } from "@/shared/api/use-api-action";
import { focusIfLost } from "@/shared/lib/focus";
import { Button } from "@/shared/ui/button";
import { ConfirmButton } from "@/shared/ui/confirm-button";
import { FormActions } from "@/shared/ui/form-actions";
import { FormError } from "@/shared/ui/form-error";
import { Notice } from "@/shared/ui/notice";
import styles from "./seeding-editor.module.scss";
import { SeedingRow } from "./seeding-row";

type Participant = components["schemas"]["Participant"];

type Props = {
  tenant: string;
  tournament: string;
  /** Os elegíveis na ordem que vale agora. */
  initial: Participant[];
};

/** "Sortear" volta a existir depois do refresh: o foco vai para ele. */
const SHUFFLE_ID = "seeding-shuffle";

/**
 * A ordem do seeding: digitar a posição (Enter ou sair do campo leva direto
 * para lá), subir e descer com as setas, sortear (pergunta antes quando a
 * ordem foi mudada, porque descarta a mudança) ou salvar e voltar para a aba
 * Organização.
 */
export function SeedingEditor({ tenant, tournament, initial }: Props) {
  const t = useTranslations("Participants.seeding");
  const router = useRouter();
  const save = useApiAction();
  const shuffle = useApiAction();
  const pending = save.pending || shuffle.pending;
  const [order, setOrder] = useState(initial);
  const [focus, setFocus] = useState<{ id: number; n: number }>();
  const params = { path: { slug: tenant, tournament } };
  const manage = `/${tenant}/${tournament}/manage`;
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

  const runShuffle = () =>
    shuffle.run(
      () =>
        api.POST("/tenants/{slug}/tournaments/{tournament}/seeding/shuffle", {
          params,
        }),
      () => {
        router.refresh();
        focusIfLost(SHUFFLE_ID);
      },
    );

  return (
    <Form
      noValidate
      onSubmit={(event) => {
        event.preventDefault();
        if (!unsaved || pending) return;
        save.run(
          () =>
            api.PUT("/tenants/{slug}/tournaments/{tournament}/seeding", {
              params,
              body: { user_ids: order.map((p) => p.user_id) },
            }),
          () => {
            router.push(manage);
            router.refresh();
          },
        );
      }}
    >
      <Stack gap={6}>
        {noneSeeded && <Notice kind="warning">{t("noneSeeded")}</Notice>}
        <ol className={styles.list} aria-label={t("listLabel")}>
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
        {changed && <Notice kind="info">{t("changed")}</Notice>}
        {(save.error ?? shuffle.error) && (
          <FormError>{save.error ?? shuffle.error}</FormError>
        )}
        <div className={styles.footer}>
          {changed ? (
            <ConfirmButton
              destructive={false}
              kind="tertiary"
              size="md"
              pending={shuffle.pending}
              question={t("shuffleConfirm")}
              onConfirm={runShuffle}
            >
              {t("shuffle")}
            </ConfirmButton>
          ) : (
            <Button
              id={SHUFFLE_ID}
              kind="tertiary"
              pending={shuffle.pending}
              onClick={runShuffle}
            >
              {t("shuffle")}
            </Button>
          )}
          <FormActions
            submitLabel={t("save")}
            pending={save.pending}
            disabled={!unsaved}
            cancelHref={manage}
          />
        </div>
      </Stack>
    </Form>
  );
}
