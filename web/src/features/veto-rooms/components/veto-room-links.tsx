"use client";

import { Launch } from "@carbon/icons-react";
import { useTranslations } from "next-intl";

import { useIsClient } from "@/shared/hooks/use-is-client";
import { Stack } from "@/shared/ui/carbon/stack";
import { CodeSnippet } from "@/shared/ui/code-snippet";
import { IconButtonLink } from "@/shared/ui/icon-button-link";
import { LocalDateTime } from "@/shared/ui/local-date-time";
import { Section } from "@/shared/ui/section";
import { useSeatLabels } from "../hooks/use-seat-labels";
import { useVetoRoom } from "../hooks/use-veto-room";
import styles from "./veto-room-links.module.scss";

/**
 * Os links da sala, para a gestão: um por lugar (centrada no capitão, um por
 * time), com o nome de quem entrou, e o de espectador, cada um com o botão de
 * copiar do Carbon e o de abrir numa guia nova. O da gestão fica de fora: é o
 * endereço desta página. Os copiados vão sem idioma: o proxy do next-intl manda
 * cada pessoa para o seu.
 */
export function VetoRoomLinks() {
  const t = useTranslations("VetoRoom.links");
  const labels = useSeatLabels();
  const { room } = useVetoRoom();
  const isClient = useIsClient();
  const links = room.links;
  if (!links) return null;
  const origin = isClient ? window.location.origin : "";
  const row = (label: string, token: string, note?: string) => (
    <div key={token} className={styles.row}>
      <p className={styles.label}>
        {label}
        {note && <span className={styles.note}> · {note}</span>}
      </p>
      <div className={styles.field}>
        <CodeSnippet type="single" aria-label={label}>
          {`${origin}/veto/${token}`}
        </CodeSnippet>
        <IconButtonLink href={`/veto/${token}`} label={t("open")} newTab>
          <Launch />
        </IconButtonLink>
      </div>
    </div>
  );

  return (
    <Section title={t("title")}>
      <Stack gap={5}>
        {room.seats.map((seat) => {
          const link = links.seats.find((l) => l.seat === seat.id);
          return (
            link &&
            row(
              room.team_size === 1
                ? labels.side(seat.side)
                : room.who_acts === "captain"
                  ? t("captain", { side: labels.side(seat.side) })
                  : labels.seat(seat),
              link.token,
              seat.name ?? t("notJoined"),
            )
          );
        })}
        {row(t("spectator"), links.spectator)}
      </Stack>
      <p className={styles.expires}>
        {t.rich("expires", {
          date: () => <LocalDateTime value={room.expires_at} />,
        })}
      </p>
    </Section>
  );
}
