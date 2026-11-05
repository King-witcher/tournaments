"use client";

import {
  type ReactNode,
  useLayoutEffect,
  useMemo,
  useRef,
  useState,
} from "react";

import { VetoSounds } from "@/features/matches/components/veto-sounds";
import { useMineMark } from "@/features/matches/hooks/use-mine-mark";
import {
  type VetoActions,
  VetoBoardContext,
} from "@/features/matches/hooks/use-veto-board";
import { useVetoRunner } from "@/features/matches/hooks/use-veto-runner";
import { isNewer } from "@/features/matches/lib/veto";
import { afterStep } from "@/features/matches/lib/veto-optimistic";
import { callApi } from "@/shared/api/call";
import { api } from "@/shared/api/client";
import type { components } from "@/shared/api/schema";
import { serverTime } from "@/shared/lib/server-clock";
import { useRoomSocket } from "../hooks/use-room-socket";
import { useSeatLabels } from "../hooks/use-seat-labels";
import { VetoRoomContext } from "../hooks/use-veto-room";
import { sideNames, withSeatNames } from "../lib/seat-names";

type Schemas = components["schemas"];

type Props = {
  token: string;
  /** A sala como o servidor a desenhou. */
  initial: Schemas["VetoRoomDetail"];
  children: ReactNode;
};

/**
 * Guarda a sala e a mantém ao vivo: começa com a do servidor, troca pela
 * resposta de cada ação (antes dela, pelo passo marcado na hora) e busca de novo
 * a cada aviso do socket, direto do navegador. O `router.refresh()` da série
 * passaria pelo servidor do Next, e um espectador custaria uma renderização a
 * cada passo. Vale a que a API montou por último (`now`). Os tabuleiros do veto
 * são os da série e leem o `VetoBoard`. A gestão não age, fora começar sem esperar
 * (`canManage`; o desfazer a sala não tem).
 */
export function VetoRoomProvider({ token, initial, children }: Props) {
  const [room, setRoom] = useState(initial);
  const [gone, setGone] = useState(false);
  const [optimistic, setOptimistic] = useState<Schemas["VetoView"] | null>(
    null,
  );
  const labels = useSeatLabels();
  const mineMark = useMineMark();
  const adopt = (next: Schemas["VetoRoomDetail"]) =>
    setRoom((current) => (isNewer(next, current) ? next : current));
  const params = { path: { token } };

  const refresh = async () => {
    const outcome = await callApi(api.GET("/veto-rooms/{token}", { params }));
    if (outcome.ok) adopt(outcome.data);
    else if (outcome.code === "veto_room.not_found") setGone(true);
  };
  const socket = useRoomSocket({
    token,
    spectator: room.role === "spectator",
    onChange: refresh,
    onGone: () => setGone(true),
  });
  const { pending, error, run } = useVetoRunner({
    initial: room,
    shown: room,
    adopt: (next) => {
      adopt(next);
      setOptimistic(null);
    },
    preview: setOptimistic,
    headers: socket.headers,
    // A resposta já é a sala inteira: só o erro precisa buscar a tela que ficou
    // para trás. Assim, um clique é um pedido só.
    refresh: (ok) => {
      if (!ok) refresh();
    },
  });
  const seat = room.seat?.name ? room.seat : null;
  const names = sideNames(room, labels);
  const veto = withSeatNames(optimistic ?? room.veto, room.seats, labels);

  // As ações leem a sala da última renderização na hora do clique: são as
  // mesmas entre renderizações, e os tiles do tabuleiro (`memo`) não
  // renderizam de novo por elas.
  const latest = useRef({ room, seat, veto, run });
  useLayoutEffect(() => {
    latest.current = { room, seat, veto, run };
  });
  const { markMine } = mineMark;
  const actions = useMemo<VetoActions>(() => {
    const params = { path: { token } };
    const go: typeof run = (...args) => {
      markMine();
      return latest.current.run(...args);
    };
    return {
      step: (choice) => {
        const { room, seat, veto } = latest.current;
        const preview = seat
          ? afterStep(veto, choice, {
              mine: seat.side,
              bestOf: room.best_of,
              bansPerSeries: room.bans_per_series,
              order: room.order,
              viewer: { id: seat.id, nickname: seat.name ?? "" },
              now: serverTime(),
            })
          : undefined;
        go(
          "step",
          ({ headers, step }) =>
            api.POST("/veto-rooms/{token}/steps", {
              params,
              headers,
              body: { ...choice, step },
            }),
          preview,
        );
      },
      ready: () =>
        go("ready", ({ headers }) =>
          api.POST("/veto-rooms/{token}/ready", { params, headers }),
        ),
      start: () =>
        go("start", ({ headers }) =>
          api.POST("/veto-rooms/{token}/begin", { params, headers }),
        ),
      // A sala não tem desfazer.
      undo: () => {},
      request: () => {},
      answer: () => {},
    };
  }, [token, markMine]);

  return (
    <VetoRoomContext
      value={{
        token,
        room,
        gone,
        adopt,
        headers: socket.headers,
        isMine: mineMark.isMine,
        clearMine: mineMark.clearMine,
      }}
    >
      <VetoBoardContext
        value={{
          veto,
          now: room.now,
          mine: seat?.side ?? null,
          bestOf: room.best_of,
          names,
          viewerId: seat?.id ?? null,
          canManage: room.role === "manager",
          hasUndo: false,
          hideEmptyMaps: true,
          pending,
          error,
          actions,
          refresh,
        }}
      >
        <VetoSounds />
        {children}
      </VetoBoardContext>
    </VetoRoomContext>
  );
}
