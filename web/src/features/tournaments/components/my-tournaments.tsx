import { getTranslations } from "next-intl/server";

import { hasSessionCookie } from "@/features/auth/server";
import { serverApi } from "@/shared/api/server";
import { Accordion } from "@/shared/ui/accordion";
import { EmptyState } from "@/shared/ui/empty-state";
import { Section } from "@/shared/ui/section";
import { TileGrid } from "@/shared/ui/tile-grid";
import { MyTournamentTile } from "./my-tournament-tile";

/** Quantos encerrados aparecem (os mais recentes). */
const FINISHED_SHOWN = 5;

/**
 * Na home, os torneios em que quem vê está inscrito, de todas as
 * comunidades, com a situação dele e a próxima partida: o caminho curto até
 * o check-in e o envio dos mapas. Os encerrados recentes ficam num bloco
 * fechado embaixo. Sem nenhum (ou sem login), não mostra nada. Fica em
 * `<Suspense>`.
 */
export async function MyTournaments() {
  if (!(await hasSessionCookie())) return null;
  const api = await serverApi();
  const entries = await api
    .GET("/me/tournaments")
    .then(({ data }) => data ?? [])
    .catch(() => []);
  if (entries.length === 0) return null;

  const active = entries.filter((e) => e.tournament.status !== "finished");
  const finished = entries
    .filter((e) => e.tournament.status === "finished")
    .slice(0, FINISHED_SHOWN);
  const t = await getTranslations("Home.tournaments");
  const key = (e: (typeof entries)[number]) =>
    `${e.tenant_slug}/${e.tournament.slug}`;

  return (
    <Section title={t("title")}>
      {active.length === 0 ? (
        <EmptyState>{t("noneActive")}</EmptyState>
      ) : (
        <TileGrid>
          {active.map((entry) => (
            <MyTournamentTile key={key(entry)} entry={entry} />
          ))}
        </TileGrid>
      )}
      {finished.length > 0 && (
        <Accordion title={t("finished")}>
          <TileGrid>
            {finished.map((entry) => (
              <MyTournamentTile key={key(entry)} entry={entry} />
            ))}
          </TileGrid>
        </Accordion>
      )}
    </Section>
  );
}
