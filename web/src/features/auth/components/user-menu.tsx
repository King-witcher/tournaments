import { Link } from "@/i18n/navigation";
import { Avatar } from "@/shared/ui/avatar";
import { getCurrentUser } from "../server";
import { LoginButton } from "./login-button";
import { LogoutButton } from "./logout-button";

/** Usuário logado (avatar, nome que leva à conta, sair) ou botão de entrar. Fica em `<Suspense>`. */
export async function UserMenu() {
  const user = await getCurrentUser();
  if (!user) return <LoginButton />;

  return (
    <div className="flex items-center gap-3">
      {user.avatar_url && <Avatar src={user.avatar_url} />}
      <Link
        href="/account"
        className="text-sm font-medium underline underline-offset-4 hover:decoration-2"
      >
        {user.display_name}
      </Link>
      <LogoutButton />
    </div>
  );
}
