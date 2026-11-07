import { getTranslations } from "next-intl/server";

import { getMyTenants } from "@/features/tenants/server";
import { MyTaskTile } from "@/features/tournaments/components/my-task-tile";
import {
  byUrgency,
  waitsForViewer,
} from "@/features/tournaments/lib/my-tournaments";
import { getMyTournaments } from "@/features/tournaments/server";
import { serverApi } from "@/shared/api/server";
import { Avatar } from "@/shared/ui/avatar";
import { Section } from "@/shared/ui/section";
import { TileGrid } from "@/shared/ui/tile-grid";
import { ContestedGameTile } from "./contested-game-tile";
import { PendingTiesTile } from "./pending-ties-tile";
import { StartDueTile } from "./start-due-tile";

/**
 * O topo da home: o que só anda se quem vê agir, em todas as comunidades.
 * Primeiro o que ele tem de fazer como jogador, que corre contra o relógio, e
 * depois o que espera a organização (os mapas contestados, em que há uma
 * partida parada, os torneios na hora de iniciar e os que esperam a decisão
 * de um empate na classificação). Nada esperando, nada
 * aparece: o aviso com som e o push já chamam quando chega algo, e a home
 * recarrega sozinha pelo canal da pessoa. Fica em `<Suspense>`.
 *
 * `focusAfter`: o alvo do foco quando o check-in feito aqui some da lista.
 */
export async function WaitingForYou({ focusAfter }: { focusAfter: string }) {
  const [tenants, mine] = await Promise.all([
    getMyTenants(),
    getMyTournaments(),
  ]);
  if (!tenants) return null;
  // Só quem organiza pergunta à API: na home de quem só joga, a rota
  // custaria mais uma leitura da sessão (Redis) a cada recarga.
  const api = await serverApi();
  const organizing = tenants.some((tenant) => tenant.role !== "player")
    ? await api
        .GET("/me/organizing")
        .then(({ data }) => data)
        .catch(() => undefined)
    : undefined;

  const tasks = (mine?.registered ?? [])
    .filter(waitsForViewer)
    .toSorted(byUrgency);
  const contested = organizing?.contested ?? [];
  const startDue = organizing?.start_due ?? [];
  const pendingTies = organizing?.pending_ties ?? [];
  const total =
    tasks.length + contested.length + startDue.length + pendingTies.length;
  if (total === 0) return null;
  const t = await getTranslations("Home.waiting");

  // Com algum logo, todas levam um (a inicial na que não tem), e os nomes
  // ficam alinhados; sem nenhum, nenhuma. Os logos vêm da lista de comunidades,
  // que a home já busca: as pendências da organização não trazem o logo.
  const shown = new Set([
    ...tasks.map((entry) => entry.tenant_slug),
    ...contested.map((item) => item.tournament.tenant_slug),
    ...startDue.map((item) => item.tournament.tenant_slug),
    ...pendingTies.map((item) => item.tournament.tenant_slug),
  ]);
  const logos = new Map(
    tenants.map((tenant) => [tenant.slug, tenant.logo_url]),
  );
  const withLogos = [...shown].some((slug) => logos.get(slug));
  const avatar = (slug: string, name: string) =>
    withLogos && (
      <Avatar shape="square" size="md" src={logos.get(slug)} name={name} />
    );

  return (
    <Section title={t("title")}>
      <TileGrid stacked>
        {tasks.map((entry) => (
          <MyTaskTile
            key={`${entry.tenant_slug}/${entry.tournament.slug}`}
            entry={entry}
            avatar={avatar(entry.tenant_slug, entry.tenant_name)}
            focusAfter={focusAfter}
          />
        ))}
        {contested.map((item) => (
          <ContestedGameTile
            key={`${item.tournament.tenant_slug}/${item.tournament.slug}/${item.key}/${item.game}`}
            item={item}
            avatar={avatar(
              item.tournament.tenant_slug,
              item.tournament.tenant_name,
            )}
          />
        ))}
        {startDue.map((item) => (
          <StartDueTile
            key={`${item.tournament.tenant_slug}/${item.tournament.slug}`}
            item={item}
            avatar={avatar(
              item.tournament.tenant_slug,
              item.tournament.tenant_name,
            )}
          />
        ))}
        {pendingTies.map((item) => (
          <PendingTiesTile
            key={`${item.tournament.tenant_slug}/${item.tournament.slug}`}
            item={item}
            avatar={avatar(
              item.tournament.tenant_slug,
              item.tournament.tenant_name,
            )}
          />
        ))}
      </TileGrid>
    </Section>
  );
}
