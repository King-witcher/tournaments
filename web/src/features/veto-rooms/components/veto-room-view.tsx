import { hasSessionCookie } from "@/features/auth/server";
import { getVetoRoom } from "../server";
import { VetoRoomGone } from "./veto-room-gone";
import { VetoRoomLive } from "./veto-room-live";
import { VetoRoomProvider } from "./veto-room-provider";

/**
 * A sala do link `token`, buscada no servidor para a primeira tela; depois,
 * ao vivo no navegador. Fica em `<Suspense>`. Sem sala, o caminho para criar
 * outra.
 */
export async function VetoRoomView({ token }: { token: string }) {
  const room = await getVetoRoom(token);
  if (!room) return <VetoRoomGone />;
  const signedIn = await hasSessionCookie();
  return (
    <VetoRoomProvider token={token} initial={room}>
      <VetoRoomLive signedIn={signedIn} />
    </VetoRoomProvider>
  );
}
