import { getTranslations } from "next-intl/server";

import { getCurrentUser } from "../server";
import { AnonymousMenu } from "./anonymous-menu";
import { UserMenuPanel } from "./user-menu-panel";

/**
 * O lugar da pessoa no cabeçalho: logado, o avatar abre o painel com Conta e
 * Sair; sem sessão, Entrar e Criar conta. Lê a sessão: fica em `<Suspense>`.
 */
export async function UserMenu() {
  const user = await getCurrentUser();
  if (!user) return <AnonymousMenu />;
  const t = await getTranslations("Auth");
  return (
    <UserMenuPanel
      label={t("menu", { name: user.display_name })}
      name={user.display_name}
      avatarSrc={user.avatar_url}
    />
  );
}
