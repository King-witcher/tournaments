"use client";

import { useTranslations } from "next-intl";
import { useState } from "react";
import { sortByRating } from "@/features/ratings/lib/sort-by-rating";
import { useRouter } from "@/i18n/navigation";
import { api } from "@/shared/api/client";
import type { components } from "@/shared/api/schema";
import { useApiAction } from "@/shared/api/use-api-action";
import { focusIfLost } from "@/shared/lib/focus";
import { ActionWithNote } from "@/shared/ui/action-with-note";
import { Button } from "@/shared/ui/button";
import { Form } from "@/shared/ui/carbon/form";
import { Stack } from "@/shared/ui/carbon/stack";
import { ConfirmButton } from "@/shared/ui/confirm-button";
import { FormActions } from "@/shared/ui/form-actions";
import { FormError } from "@/shared/ui/form-error";
import { LeaveGuard } from "@/shared/ui/leave-guard";
import { Notice } from "@/shared/ui/notice";
import { Panel } from "@/shared/ui/panel";
import { Section } from "@/shared/ui/section";
import styles from "./seeding-editor.module.scss";
import { SeedingList } from "./seeding-list";

type Team = components["schemas"]["Team"];

type Props = {
  tenant: string;
  tournament: string;
  /** O título da seção: quantos entram. */
  title: string;
  /** O que a ordem decide, pelo formato da primeira fase. */
  description: string;
  /** Os times elegíveis na ordem que vale agora. */
  initial: Team[];
  /** Rating de cada time que tem (a média dos jogadores), pelo id do time. */
  ratings: ReadonlyMap<number, number>;
};

/** "Sortear" volta a existir depois do refresh: o foco vai para ele. */
const SHUFFLE_ID = "seeding-shuffle";

/** A identidade da ordem que a API mandou, sem contar as outras mudanças do torneio. */
function orderKey(order: Team[]) {
  return order.map((team) => `${team.id}:${team.seed}`).join(",");
}

/**
 * A ordem da chave, editável: arrastar pela alça, sortear (pergunta antes
 * quando a ordem foi mudada, porque descarta a mudança), ordenar por rating
 * (preenche a ordem na tela: maior rating primeiro, quem não tem no fim, e
 * fica desabilitado, com o motivo, sem nenhum rating) e salvar. Recomeça do
 * que a API devolveu quando a ordem dela muda; outras mudanças do torneio
 * (o tempo real) não apagam o que a pessoa arrastou.
 */
export function SeedingEditor({
  tenant,
  tournament,
  title,
  description,
  initial,
  ratings,
}: Props) {
  const t = useTranslations("Participants.seeding");
  const tRatings = useTranslations("Ratings.seeding");
  const router = useRouter();
  const save = useApiAction();
  const shuffle = useApiAction();
  const pending = save.pending || shuffle.pending;
  const [order, setOrder] = useState(initial);
  const [saved, setSaved] = useState(false);
  const [shown, setShown] = useState(() => orderKey(initial));
  if (orderKey(initial) !== shown) {
    setShown(orderKey(initial));
    setOrder(initial);
  }
  const params = { path: { slug: tenant, tournament } };
  const changed = order.some((team, i) => team.id !== initial[i]?.id);
  // Ninguém tem seed: iniciar assim sorteia, e a ordem da tela não vale.
  const noneSeeded = initial.every((p) => p.seed == null);
  // Sem seed nenhum, a lista é a ordem de inscrição e não leva números; eles
  // aparecem quando a pessoa mexe na ordem.
  const numbered = !noneSeeded || changed;
  // Ainda há quem não tem seed: salvar grava a ordem que aparece.
  const unsaved = changed || initial.some((p) => p.seed == null);
  const anyRated = initial.some((team) => ratings.has(team.id));

  function ratingText(team: Team) {
    const rating = ratings.get(team.id);
    return rating === undefined
      ? tRatings("unrated")
      : tRatings("rating", { rating });
  }

  function reorder(next: Team[]) {
    if (pending) return;
    setOrder(next);
    setSaved(false);
  }

  const runShuffle = () =>
    shuffle.run(
      () =>
        api.POST("/tenants/{slug}/tournaments/{tournament}/seeding/shuffle", {
          params,
        }),
      () => {
        setSaved(false);
        router.refresh();
        focusIfLost(SHUFFLE_ID);
      },
    );

  return (
    <Section
      title={title}
      description={description}
      action={
        <span className={styles.actions}>
          <ActionWithNote
            compact
            blocked={!anyRated}
            note={anyRated ? undefined : tRatings("sortUnavailable")}
          >
            <Button
              kind="tertiary"
              size="sm"
              disabled={pending || !anyRated}
              onClick={() => reorder(sortByRating(order, ratings))}
            >
              {tRatings("sort")}
            </Button>
          </ActionWithNote>
          {changed ? (
            <ConfirmButton
              destructive={false}
              kind="tertiary"
              size="sm"
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
              size="sm"
              pending={shuffle.pending}
              onClick={runShuffle}
            >
              {t("shuffle")}
            </Button>
          )}
        </span>
      }
    >
      <Panel>
        <LeaveGuard
          unsaved={changed && !saved}
          path={`/${tenant}/${tournament}/seeding`}
        />
        <Form
          noValidate
          onSubmit={(event) => {
            event.preventDefault();
            if (!unsaved || saved || pending) return;
            save.run(
              () =>
                api.PUT("/tenants/{slug}/tournaments/{tournament}/seeding", {
                  params,
                  body: { team_ids: order.map((team) => team.id) },
                }),
              () => {
                setSaved(true);
                router.refresh();
              },
            );
          }}
        >
          <Stack gap={6}>
            {noneSeeded && !saved && (
              <Notice kind="warning">{t("noneSeeded")}</Notice>
            )}
            <SeedingList
              order={order}
              numbered={numbered}
              showNoSeed={!noneSeeded}
              ratingText={anyRated ? ratingText : undefined}
              onChange={reorder}
            />
            {changed && !saved && <Notice>{t("changed")}</Notice>}
            {saved && <Notice kind="success">{t("saved")}</Notice>}
            {(save.error ?? shuffle.error) && (
              <FormError>{save.error ?? shuffle.error}</FormError>
            )}
            <FormActions
              submitLabel={t("save")}
              pending={save.pending}
              disabled={!unsaved || saved}
              onCancel={changed && !saved ? () => setOrder(initial) : undefined}
            />
          </Stack>
        </Form>
      </Panel>
    </Section>
  );
}
