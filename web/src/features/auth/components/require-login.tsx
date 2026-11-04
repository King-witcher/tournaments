import type { ReactNode } from "react";

import { getSession } from "../server";
import { LoginPrompt } from "./login-prompt";
import { SessionUnavailable } from "./session-unavailable";

type Props = {
  /** O que quem não está logado lê acima do botão de entrar. */
  message: string;
  children: ReactNode;
};

/**
 * Mostra `children` só para quem está logado. Com a API fora, a falha, e não o
 * convite a entrar. Fica em `<Suspense>`.
 */
export async function RequireLogin({ message, children }: Props) {
  const session = await getSession();
  if (session.kind === "signed-in") return children;
  if (session.kind === "unavailable") return <SessionUnavailable />;
  return <LoginPrompt message={message} />;
}
