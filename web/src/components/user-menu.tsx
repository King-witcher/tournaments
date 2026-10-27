import Image from "next/image";

import { serverApi } from "@/lib/api/server";
import { LoginButton } from "./login-button";
import { LogoutButton } from "./logout-button";

/** Usuário logado (avatar, nome, sair) ou botão de entrar. Fica em `<Suspense>`. */
export async function UserMenu() {
  const api = await serverApi();
  const user = await api
    .GET("/me")
    .then(({ data }) => data)
    .catch(() => undefined);

  if (!user) return <LoginButton />;

  return (
    <div className="flex items-center gap-3">
      {user.avatar_url && (
        <Image
          src={user.avatar_url}
          alt=""
          width={28}
          height={28}
          className="rounded-full"
        />
      )}
      <span className="text-sm font-medium">{user.display_name}</span>
      <LogoutButton />
    </div>
  );
}
