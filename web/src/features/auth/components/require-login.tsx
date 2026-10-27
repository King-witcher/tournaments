import type { ReactNode } from "react";

import { getCurrentUser } from "../server";
import { LoginPrompt } from "./login-prompt";

type Props = {
  /** O que quem não está logado lê acima do botão de entrar. */
  message: string;
  children: ReactNode;
};

/** Mostra `children` só para quem está logado. Fica em `<Suspense>`. */
export async function RequireLogin({ message, children }: Props) {
  const user = await getCurrentUser();
  return user ? children : <LoginPrompt message={message} />;
}
