import type { ReactNode } from "react";

import { getSession } from "../server";
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
 */
export async function HomeSessionGate({ pending, children }: Props) {
  const session = await getSession();
  if (session.kind === "signed-in") return children;
  if (session.kind === "unavailable") return <SessionUnavailable />;
  return pending;
}
