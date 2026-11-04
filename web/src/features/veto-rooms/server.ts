import type { Metadata } from "next";
import { getTranslations } from "next-intl/server";
import { cache } from "react";

import type { components } from "@/shared/api/schema";
import { serverApi } from "@/shared/api/server";
import { siteOpenGraph } from "@/shared/lib/link-preview";

/**
 * A sala do link `token`, uma busca por requisição: a página e a prévia do link
 * (`vetoRoomMetadata`) dividem o resultado. `null` se não existe ou venceu. Lê
 * cookies (o client do servidor): quem usa fica em `<Suspense>`. Qualquer outro
 * erro da API sobe para a página de erro da rota.
 */
export const getVetoRoom = cache(async (token: string) => {
  const api = await serverApi();
  const { data, error, response } = await api.GET("/veto-rooms/{token}", {
    params: { path: { token } },
  });
  if (data) return data;
  if (response.status === 404) return null;
  throw new Error(`sala de veto: ${error?.error ?? response.status}`);
});

/**
 * A prévia do link da sala (Discord, WhatsApp). Cada lugar tem o seu link, e
 * saem vários no mesmo chat: o título diz de quem é este, a descrição é o que
 * não muda (modo e melhor de), e a imagem é o ícone em miniatura. O token está
 * na URL: nada de indexar, de mandar o endereço a outro site nem de pôr o nome
 * de quem entrou (a rede guarda a prévia, e o nome é livre). O cabeçalho das
 * mesmas regras está no `next.config.ts`. Sala vencida ou API fora: o texto
 * geral.
 */
export async function vetoRoomMetadata(token: string): Promise<Metadata> {
  const [t, tNav, room] = await Promise.all([
    getTranslations("VetoRoom"),
    getTranslations("Nav"),
    getVetoRoom(token).catch(() => null),
  ]);
  const title = room ? `${t("title")} · ${linkLabel(room, t)}` : t("title");
  const description = room
    ? `${
        room.team_size === 1
          ? t("modes.duel")
          : t("modes.teams", { size: room.team_size })
      } · ${t("bestOf", { count: room.best_of })}`
    : t("description");
  return {
    title,
    description,
    robots: { index: false, follow: false },
    referrer: "no-referrer",
    openGraph: {
      ...(await siteOpenGraph()),
      title,
      description,
      images: [
        {
          url: "/icon",
          width: 512,
          height: 512,
          type: "image/png",
          alt: tNav("brand"),
        },
      ],
    },
    twitter: { card: "summary" },
  };
}

/** De quem é o link, como na lista de links da gestão (`VetoRoomLinks`). */
function linkLabel(
  room: components["schemas"]["VetoRoomDetail"],
  t: Awaited<ReturnType<typeof getTranslations<"VetoRoom">>>,
) {
  const { seat } = room;
  if (room.role !== "player" || !seat) {
    return t(`roles.${room.role as "manager" | "spectator"}`);
  }
  const side = seat.side.toUpperCase();
  if (room.team_size === 1) return t("seats.side", { side });
  return room.who_acts === "captain"
    ? t("links.captain", { side: t("seats.side", { side }) })
    : t("seats.seat", { side, position: seat.position });
}
