import { getTranslations } from "next-intl/server";

import { getTenant } from "@/features/tenants/server";
import type { components } from "@/shared/api/schema";
import { serverApi } from "@/shared/api/server";
import { Stack } from "@/shared/ui/carbon/stack";
import { EmptyState } from "@/shared/ui/empty-state";
import { Section } from "@/shared/ui/section";
import { TileGrid } from "@/shared/ui/tile-grid";
import { CommunityTournamentTile } from "./community-tournament-tile";
import { FinishedTournaments } from "./finished-tournaments";
import { NewTournamentButton } from "./new-tournament-button";

type Schemas = components["schemas"];

/**
 * Do que está acontecendo para o que ainda vai começar; dentro de cada
 * status, a ordem da API (mais novo primeiro).
 */
const ORDER: Schemas["TournamentStatus"][] = [
  "in_progress",
  "check_in",
  "registration",
  "draft",
];

/**
 * A lista de torneios do início da comunidade (o único lugar que lista todos): os ativos em tiles, do que está acontecendo
 * para o que vai começar, e embaixo os encerrados, em tabela (`FinishedTournaments`).
 * O jogador só recebe os publicados (a API filtra); a organização vê os
 * rascunhos e, sem nenhum torneio, o "Criar torneio" (fora da exclusão
 * agendada, quando nada muda). Fica em `<Suspense>`.
 */
export async function TournamentList({ slug }: { slug: string }) {
  const api = await serverApi();
  const [loaded, t, tournaments] = await Promise.all([
    getTenant(slug),
    getTranslations("Tournaments"),
    api
      .GET("/tenants/{slug}/tournaments", { params: { path: { slug } } })
      .then(({ data }) => data ?? []),
  ]);
  // Sem acesso, o cabeçalho da seção já mostra o erro.
  if (loaded.kind !== "ok") return null;
  const { tenant } = loaded;
  const organizer = tenant.role !== "player" && !tenant.deletion_scheduled_for;
  const active = tournaments
    .filter((x) => x.status !== "finished")
    .toSorted((a, b) => ORDER.indexOf(a.status) - ORDER.indexOf(b.status));
  const finished = tournaments.filter((x) => x.status === "finished");
  // Numa grade com alguma capa, todo tile leva o espaço dela (com o logo ou a
  // inicial da comunidade no lugar da que falta); sem nenhuma, nenhum leva.
  const media = active.some((x) => x.cover_url);
  const create = organizer && (
    // O "Novo torneio" primário já está no cabeçalho.
    <NewTournamentButton tenant={slug} kind="tertiary">
      {t("create")}
    </NewTournamentButton>
  );

  if (tournaments.length === 0) {
    return (
      <EmptyState action={create}>
        {organizer ? t("empty") : t("emptyPlayer")}
      </EmptyState>
    );
  }

  return (
    <Stack gap={7}>
      <Section title={t("active")}>
        {active.length === 0 ? (
          <EmptyState action={create}>{t("noneActive")}</EmptyState>
        ) : (
          <TileGrid>
            {active.map((tournament) => (
              <CommunityTournamentTile
                key={tournament.slug}
                tenant={{
                  slug,
                  name: tenant.name,
                  logo_url: tenant.logo_url,
                }}
                tournament={tournament}
                media={media}
              />
            ))}
          </TileGrid>
        )}
      </Section>
      <FinishedTournaments slug={slug} tournaments={finished} />
    </Stack>
  );
}
