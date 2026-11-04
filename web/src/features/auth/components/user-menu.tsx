import { getTranslations } from "next-intl/server";

import { getSession } from "../server";
import { AnonymousMenu } from "./anonymous-menu";
import { ClearExpiredSession } from "./clear-expired-session";
import { SessionUnavailableMenu } from "./session-unavailable-menu";
import { UserMenuPanel } from "./user-menu-panel";

/**
 * O lugar da pessoa no cabeçalho, presente em toda página: logado, o avatar e
 * o nome abrem o painel com Conta e Sair; sem sessão, Entrar e Criar conta. O
 * cookie que a API recusou sai daqui (`ClearExpiredSession`); com a API fora,
 * nada de "Entrar". Lê a sessão: fica em `<Suspense>`.
 */
export async function UserMenu() {
  const session = await getSession();
  if (session.kind === "unavailable") return <SessionUnavailableMenu />;
  if (session.kind !== "signed-in") {
    return (
      <>
        <AnonymousMenu />
        {session.kind === "expired" && <ClearExpiredSession />}
      </>
    );
  }
  const { user } = session;
  const t = await getTranslations("Auth");
  return (
    <UserMenuPanel
      label={t("menu", { name: user.display_name })}
      name={user.display_name}
      avatarSrc={user.avatar_url}
    />
  );
}
