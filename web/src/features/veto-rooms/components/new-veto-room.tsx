import { hasSessionCookie } from "@/features/auth/server";
import { serverApi } from "@/shared/api/server";
import { NewVetoRoomForm } from "./new-veto-room-form";

/** O formulário com os mapas ativos e se quem vê tem sessão. Fica em `<Suspense>`. */
export async function NewVetoRoom() {
  const [api, signedIn] = await Promise.all([serverApi(), hasSessionCookie()]);
  const { data: maps } = await api.GET("/maps");
  return <NewVetoRoomForm maps={maps ?? []} signedIn={signedIn} />;
}
