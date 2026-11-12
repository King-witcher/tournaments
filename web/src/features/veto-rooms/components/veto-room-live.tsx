"use client";

import { useTranslations } from "next-intl";

import { VetoHistory } from "@/features/matches/components/veto-history";
import { VetoReady } from "@/features/matches/components/veto-ready";
import { VetoSoundToggle } from "@/features/matches/components/veto-sound-toggle";
import { useVetoBoard } from "@/features/matches/hooks/use-veto-board";
import { Accordion } from "@/shared/ui/carbon/accordion";
import { PageColumns } from "@/shared/ui/page-columns";
import { PageHeader } from "@/shared/ui/page-header";
import { useVetoRoom } from "../hooks/use-veto-room";
import { VetoRoomGone } from "./veto-room-gone";
import { VetoRoomJoin } from "./veto-room-join";
import { VetoRoomLinks } from "./veto-room-links";
import { VetoRoomSteps } from "./veto-room-steps";

/**
 * A página da sala, que se adapta ao link: quem joga entra no lugar e faz os
 * passos; o espectador assiste; a gestão vê os links e o andamento (antes do
 * começo, os links são a tarefa e vêm na coluna principal; depois, o veto
 * ocupa a largura toda, como no link de quem joga, e os links ficam embaixo).
 */
export function VetoRoomLive({ signedIn }: { signedIn: boolean }) {
  const t = useTranslations("VetoRoom");
  const { veto, names } = useVetoBoard();
  const { room, gone } = useVetoRoom();
  if (gone) return <VetoRoomGone />;

  const mode =
    room.team_size === 1
      ? t("modes.duel")
      : t("modes.teams", { size: room.team_size });
  const who =
    room.role === "player"
      ? room.seat?.name
      : t(`roles.${room.role as "manager" | "spectator"}`);
  const history = veto && veto.history.length > 0 && (
    <Accordion>
      <VetoHistory veto={veto} names={names} />
    </Accordion>
  );
  const waiting = room.veto.status === "waiting_ready";
  const needsName = room.role === "player" && !room.seat?.name;

  return (
    <>
      <PageHeader
        title={t("title")}
        meta={[mode, t("bestOf", { count: room.best_of }), who]
          .filter(Boolean)
          .join(" · ")}
        actions={<VetoSoundToggle />}
      />
      {needsName ? (
        <VetoRoomJoin signedIn={signedIn} />
      ) : room.role !== "manager" ? (
        <>
          <VetoRoomSteps />
          {history}
        </>
      ) : waiting ? (
        <PageColumns main={<VetoRoomLinks />} aside={<VetoReady room />} />
      ) : (
        <>
          <VetoRoomSteps />
          {history}
          <VetoRoomLinks />
        </>
      )}
    </>
  );
}
