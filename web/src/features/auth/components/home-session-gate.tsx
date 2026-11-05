import type { ReactNode } from "react";

import { getMyTenants } from "@/features/tenants/server";
import { getMyTournaments } from "@/features/tournaments/server";
import { preload } from "@/shared/lib/preload";
import { getLogins, getSession } from "../server";
import { SessionUnavailable } from "./session-unavailable";

type Props = {
  /** O esqueleto da página, enquanto o cookie recusado sai. */
  pending: ReactNode;
  children: ReactNode;
};

/**
 * O início de quem entrou (`/me`), que o proxy serve em `/` só pela presença do
 * cookie: as listas só com a sessão confirmada. Com o cookie recusado, a espera
 * continua até o `ClearExpiredSession` levar à apresentação; com a API fora, a
 * falha. Fica em `<Suspense>`.
 *
 * As buscas das listas (comunidades, torneios, logins) saem junto com a da
 * sessão, não depois: dependem só do cookie, que o proxy já viu. Com o cookie
 * recusado, voltam 401 e ficam sem uso.
 */
export async function HomeSessionGate({ pending, children }: Props) {
  preload(getMyTenants());
  preload(getMyTournaments());
  preload(getLogins());
  const session = await getSession();
  if (session.kind === "signed-in") return children;
  if (session.kind === "unavailable") return <SessionUnavailable />;
  return pending;
}
