import { getTranslations } from "next-intl/server";

import { getMyTenants } from "@/features/tenants/server";
import { Accordion } from "@/shared/ui/accordion";
import { EmptyState } from "@/shared/ui/empty-state";
import { Section } from "@/shared/ui/section";
import { TileGrid } from "@/shared/ui/tile-grid";
import { splitMine } from "../lib/my-tournaments";
import { getMyTournaments } from "../server";
import { MyTournamentTile } from "./my-tournament-tile";

/**
 * Na home, os torneios em que quem vê está inscrito, de todas as comunidades,
 * com a situação dele e a próxima série, do que espera por ele ao que só
 * espera a data. Os em que a participação dele acabou ficam num bloco fechado
 * embaixo. Sem nenhum, a seção só aparece, com o estado vazio, se também não
 * há nada aberto: as "Inscrições abertas" já dizem o que fazer. Quem não
 * participa de comunidade nenhuma não a vê (vê o `NoTenants`). Fica em
 * `<Suspense>`.
 */
export async function MyTournaments() {
  const [tenants, mine] = await Promise.all([
    getMyTenants(),
    getMyTournaments(),
  ]);
  if (!mine || !tenants?.length) return null;
  if (mine.registered.length === 0 && mine.open.length > 0) return null;

  const { active, over } = splitMine(mine.registered);
  const t = await getTranslations("Home.tournaments");
  const key = (e: (typeof active)[number]) =>
    `${e.tenant_slug}/${e.tournament.slug}`;
  // Numa grade com alguma capa ou logo, todo tile leva o espaço da capa (com
  // o logo ou a inicial no lugar da que falta); sem nenhuma, nenhum leva.
  const media = (list: typeof active) =>
    list.some(
      (e) => e.tournament.cover_url || e.tenant_cover_url || e.tenant_logo_url,
    );
  const activeMedia = media(active);
  const overMedia = media(over);

  return (
    <Section title={t("title")}>
      {active.length === 0 ? (
        <EmptyState>{over.length > 0 ? t("noneActive") : t("none")}</EmptyState>
      ) : (
        <TileGrid>
          {active.map((entry) => (
            <MyTournamentTile
              key={key(entry)}
              entry={entry}
              media={activeMedia}
            />
          ))}
        </TileGrid>
      )}
      {over.length > 0 && (
        <Accordion title={t("over")}>
          <TileGrid>
            {over.map((entry) => (
              <MyTournamentTile
                key={key(entry)}
                entry={entry}
                media={overMedia}
              />
            ))}
          </TileGrid>
        </Accordion>
      )}
    </Section>
  );
}
