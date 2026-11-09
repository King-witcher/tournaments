import { getTranslations } from "next-intl/server";

import { Section } from "@/shared/ui/section";
import { TileGrid } from "@/shared/ui/tile-grid";
import { getMyTournaments } from "../server";
import { OpenTournamentTile } from "./open-tournament-tile";

/**
 * Na home, os torneios das comunidades de quem vê em que ele pode se inscrever
 * agora pelo botão. Nenhum, nada aparece. Fica em `<Suspense>`.
 */
export async function OpenTournaments() {
  const mine = await getMyTournaments();
  if (!mine?.open.length) return null;
  const t = await getTranslations("Home.open");
  // Como em "Seus torneios": com alguma capa ou logo, todo tile leva o espaço.
  const media = mine.open.some(
    (e) => e.tournament.cover_url || e.tenant_cover_url || e.tenant_logo_url,
  );

  return (
    <Section title={t("title")}>
      <TileGrid>
        {mine.open.map((entry) => (
          <OpenTournamentTile
            key={`${entry.tenant_slug}/${entry.tournament.slug}`}
            entry={entry}
            media={media}
          />
        ))}
      </TileGrid>
    </Section>
  );
}
