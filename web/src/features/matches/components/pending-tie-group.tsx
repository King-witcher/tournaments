"use client";

import { ArrowDown, ArrowUp } from "@carbon/icons-react";
import { useTranslations } from "next-intl";
import { useEffect, useRef, useState } from "react";

import { useRouter } from "@/i18n/navigation";
import { api } from "@/shared/api/client";
import { useApiAction } from "@/shared/api/use-api-action";
import { Stack } from "@/shared/ui/carbon/stack";
import { ConfirmButton } from "@/shared/ui/confirm-button";
import { ContainedList, ContainedListItem } from "@/shared/ui/contained-list";
import { ErrorText } from "@/shared/ui/error-text";
import { IconButton } from "@/shared/ui/icon-button";
import { TextInput } from "@/shared/ui/text-input";
import styles from "./pending-tie-group.module.scss";

type Props = {
  tenant: string;
  slug: string;
  /** A melhor posição do grupo. */
  position: number;
  /** Os empatados, na ordem da chave. */
  teams: { id: number; name: string }[];
  /** É o único pendente: decidir encerra o torneio (o Modal diz). */
  last: boolean;
};

/** Até onde vai o motivo (a API recusa acima disso). */
const MAX_REASON = 280;

/**
 * Um empate do fim que espera a organização: os empatados numa lista, com
 * subir e descer em cada um (setas, como os desempates do formulário:
 * funcionam igual no teclado e no toque, e o grupo é pequeno), o motivo
 * opcional e os botões "Salvar ordem" e "Usar a ordem da chave", que confirmam
 * antes, porque a ordem gravada não muda. Depois de mover, o foco fica na
 * mesma seta, no lugar novo da linha.
 */
export function PendingTieGroup({
  tenant,
  slug,
  position,
  teams,
  last,
}: Props) {
  const t = useTranslations("Bracket.ties");
  const router = useRouter();
  const { run, pending, error } = useApiAction();
  const [order, setOrder] = useState(teams.map((x) => x.id));
  const [reason, setReason] = useState("");
  const [focus, setFocus] = useState<{ id: number; dir: "up" | "down" }>();
  const buttons = useRef(new Map<string, HTMLButtonElement | null>());
  const nameOf = new Map(teams.map((x) => [x.id, x.name]));

  useEffect(() => {
    if (!focus) return;
    const same = buttons.current.get(`${focus.id}-${focus.dir}`);
    const other = buttons.current.get(
      `${focus.id}-${focus.dir === "up" ? "down" : "up"}`,
    );
    (same && !same.disabled ? same : other)?.focus();
  }, [focus]);

  const move = (index: number, step: -1 | 1) => {
    const next = [...order];
    [next[index], next[index + step]] = [next[index + step], next[index]];
    setOrder(next);
    setFocus({ id: order[index], dir: step === -1 ? "up" : "down" });
  };
  const decide = (teamIds: number[], withReason: boolean) =>
    run(
      () =>
        api.POST("/tenants/{slug}/tournaments/{tournament}/ties", {
          params: { path: { slug: tenant, tournament: slug } },
          body: {
            teams: teamIds,
            reason: withReason && reason.trim() ? reason.trim() : null,
          },
        }),
      () => router.refresh(),
    );
  const question = (text: string) => (last ? `${text} ${t("ends")}` : text);

  return (
    <Stack gap={5}>
      <ContainedList label={t("order")} size="md">
        {order.map((id, i) => {
          const name = nameOf.get(id) ?? "";
          return (
            <ContainedListItem
              key={id}
              action={
                <>
                  <IconButton
                    ref={(b) => {
                      buttons.current.set(`${id}-up`, b);
                    }}
                    label={t("up", { name })}
                    disabled={i === 0 || pending}
                    onClick={() => move(i, -1)}
                  >
                    <ArrowUp />
                  </IconButton>
                  <IconButton
                    ref={(b) => {
                      buttons.current.set(`${id}-down`, b);
                    }}
                    label={t("down", { name })}
                    disabled={i === order.length - 1 || pending}
                    onClick={() => move(i, 1)}
                  >
                    <ArrowDown />
                  </IconButton>
                </>
              }
            >
              <span className={styles.row}>
                <span className={styles.position}>
                  {t("position", { position: position + i })}
                </span>
                <span className={styles.name}>{name}</span>
              </span>
            </ContainedListItem>
          );
        })}
      </ContainedList>
      <TextInput
        labelText={t("reason")}
        helperText={t("reasonHint")}
        value={reason}
        maxLength={MAX_REASON}
        onChange={(e) => setReason(e.target.value)}
      />
      {error && <ErrorText>{error}</ErrorText>}
      <div className={styles.actions}>
        <ConfirmButton
          destructive={false}
          kind="primary"
          pending={pending}
          question={question(t("saveQuestion"))}
          onConfirm={() => decide(order, true)}
        >
          {t("save")}
        </ConfirmButton>
        <ConfirmButton
          destructive={false}
          kind="tertiary"
          pending={pending}
          question={question(t("seedQuestion"))}
          onConfirm={() =>
            decide(
              teams.map((x) => x.id),
              false,
            )
          }
        >
          {t("seed")}
        </ConfirmButton>
      </div>
    </Stack>
  );
}
