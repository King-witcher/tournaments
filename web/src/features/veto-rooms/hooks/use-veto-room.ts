"use client";

import { createContext, useContext } from "react";

import type { components } from "@/shared/api/schema";

export type VetoRoomValue = {
  token: string;
  room: components["schemas"]["VetoRoomDetail"];
  /** A sala venceu com a página aberta. */
  gone: boolean;
  /** Troca a sala pela resposta de uma ação (entrar num lugar). */
  adopt: (room: components["schemas"]["VetoRoomDetail"]) => void;
  /** Os cabeçalhos das ações. */
  headers: () => Record<string, string>;
  isMine: () => boolean;
  clearMine: () => void;
};

export const VetoRoomContext = createContext<VetoRoomValue | null>(null);

/** A sala como este link a vê. Só dentro do `VetoRoomProvider`. */
export function useVetoRoom(): VetoRoomValue {
  const value = useContext(VetoRoomContext);
  if (!value) throw new Error("useVetoRoom fora do VetoRoomProvider");
  return value;
}
