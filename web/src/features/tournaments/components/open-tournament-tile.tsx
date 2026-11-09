import { useLocale, useTranslations } from "next-intl";

import type { components } from "@/shared/api/schema";
import { LinkTile } from "@/shared/ui/link-tile";
import { LocalDateTime } from "@/shared/ui/local-date-time";
import { MetaLines } from "@/shared/ui/meta-lines";
import { useModeLabel } from "../hooks/use-mode-label";
import { localizedName } from "../lib/localize";
import { TournamentMedia } from "./tournament-media";

/**
 * Um torneio aberto à inscrição de quem vê: a capa (ou o logo da comunidade
 * no lugar dela), o nome e o que pesa para decidir (quando, formato, vagas).
 * A inscrição fica na página dele, junto da descrição e das regras.
 */
export function OpenTournamentTile({
  entry,
  media,
}: {
  entry: components["schemas"]["OpenTournament"];
  /** A grade tem alguma capa ou logo: este tile também leva o espaço. */
  media: boolean;
}) {
  const t = useTranslations("Home.open");
  const tTournaments = useTranslations("Home.tournaments");
  const locale = useLocale();
  const modeLabel = useModeLabel();
  const { tournament } = entry;
  const mode = modeLabel(tournament.mode, tournament.team_size);
  const spots = t("spots", {
    count: tournament.participants,
    max: tournament.max_participants,
  });

  return (
    <LinkTile
      href={`/${entry.tenant_slug}/${tournament.slug}`}
      title={localizedName(tournament, locale)}
      media={
        media && (
          <TournamentMedia
            cover={tournament.cover_url}
            tenantCover={entry.tenant_cover_url}
            logo={entry.tenant_logo_url}
            community={entry.tenant_name}
          />
        )
      }
      meta={
        <MetaLines
          lines={[
            <>
              {entry.tenant_name}
              {" · "}
              {tournament.starts_at ? (
                <LocalDateTime value={tournament.starts_at} />
              ) : (
                tTournaments("noDate")
              )}
            </>,
            `${mode} · ${spots}`,
          ]}
        />
      }
    />
  );
}
