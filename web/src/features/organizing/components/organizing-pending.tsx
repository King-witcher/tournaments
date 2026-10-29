import { getTranslations } from "next-intl/server";

import { getMyTenants } from "@/features/tenants/server";
import { serverApi } from "@/shared/api/server";
import { Section } from "@/shared/ui/section";
import { TileGrid } from "@/shared/ui/tile-grid";
import { ContestedGameTile } from "./contested-game-tile";
import { StartDueTile } from "./start-due-tile";

/**
 * Na home, para quem é organizer ou owner em alguma comunidade: o que
 * espera a organização em todas elas, cada item como link para onde se
 * resolve (os mapas contestados primeiro: há uma partida parada). Sem nada
 * esperando, não mostra nada: o aviso com som e o push já chamam quando
 * chega algo, e a home recarrega sozinha pelo canal da pessoa. Fica em
 * `<Suspense>`.
 */
export async function OrganizingPending() {
  // Só quem organiza pergunta à API: na home de quem só joga, a rota
  // custaria mais uma leitura da sessão (Redis) a cada recarga.
  const tenants = await getMyTenants();
  if (!tenants?.some((tenant) => tenant.role !== "player")) return null;
  const api = await serverApi();
  const pending = await api
    .GET("/me/organizing")
    .then(({ data }) => data)
    .catch(() => undefined);
  if (!pending) return null;
  const { contested, start_due } = pending;
  if (contested.length === 0 && start_due.length === 0) return null;
  const t = await getTranslations("Organizing");

  return (
    <Section title={t("title")}>
      <TileGrid>
        {contested.map((item) => (
          <ContestedGameTile
            key={`${item.tournament.tenant_slug}/${item.tournament.slug}/${item.key}/${item.game}`}
            item={item}
          />
        ))}
        {start_due.map((item) => (
          <StartDueTile
            key={`${item.tournament.tenant_slug}/${item.tournament.slug}`}
            item={item}
          />
        ))}
      </TileGrid>
    </Section>
  );
}
