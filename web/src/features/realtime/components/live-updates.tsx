"use client";

import { useEffect } from "react";

import { useRealtimeHub } from "../hooks/use-realtime-hub";
import type { Interest } from "../lib/interest";

/**
 * A página diz o que mostra, e ela recarrega sozinha quando isso muda (para
 * quem está logado): `tenant` (as abas da comunidade e a pessoa nela),
 * `tenant` + `tournament` (as páginas do torneio), com `match` (a partida) e
 * `me` ("Seus torneios"); `fast` para o que anda ao vivo (o veto), que
 * recarrega com a janela curta. Assina ao montar e desfaz ao desmontar; duas
 * iguais na mesma página dão uma assinatura só. Não desenha nada.
 *
 * ```tsx
 * <LiveUpdates tenant={tenant} tournament={tournament} match={match} />
 * ```
 */
export function LiveUpdates({ tenant, tournament, match, me, fast }: Interest) {
  const hub = useRealtimeHub();
  useEffect(
    () => hub?.watch({ tenant, tournament, match, me, fast }),
    [hub, tenant, tournament, match, me, fast],
  );
  return null;
}
