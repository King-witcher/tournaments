import { getTranslations } from "next-intl/server";
import type { ReactNode } from "react";

import { PageHeader } from "@/shared/ui/page-header";
import { getSession } from "../server";
import { LoginPrompt } from "./login-prompt";
import { SessionUnavailable } from "./session-unavailable";

/**
 * As páginas da conta só para quem entrou: cada uma desenha o próprio
 * cabeçalho, com o título da seção. Sem sessão, no lugar delas, "Sua conta" e
 * o convite a entrar (que volta para a página); com a API fora, a falha.
 * Fica em `<Suspense>`.
 */
export async function AccountGate({ children }: { children: ReactNode }) {
  const session = await getSession();
  if (session.kind === "signed-in") return children;
  const t = await getTranslations("Account");
  return (
    <>
      <PageHeader title={t("title")} />
      {session.kind === "unavailable" ? (
        <SessionUnavailable />
      ) : (
        <LoginPrompt message={t("loginRequired")} />
      )}
    </>
  );
}
