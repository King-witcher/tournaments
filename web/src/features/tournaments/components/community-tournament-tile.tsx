import { useLocale, useTranslations } from "next-intl";

import type { components } from "@/shared/api/schema";
import { LinkTile } from "@/shared/ui/link-tile";
import { LocalDateTime } from "@/shared/ui/local-date-time";
import { MetaLines } from "@/shared/ui/meta-lines";
import { StatusTag } from "@/shared/ui/status-tag";
import { useModeLabel } from "../hooks/use-mode-label";
import { checkInPending } from "../lib/check-in";
import { localizedName } from "../lib/localize";
import { participantsEditable } from "../lib/status";
import { TournamentMedia } from "./tournament-media";

/**
 * Um torneio ativo na aba Torneios da comunidade: com `media`, a capa no topo
 * (ou o logo da comunidade no lugar dela), o nome, o início, a modalidade e as
 * vagas, e as etiquetas do estado, do que é de quem vê (inscrito, falta o
 * check-in) e da visibilidade.
 */
export function CommunityTournamentTile({
  tenant,
  tournament,
  media,
}: {
  tenant: {
    slug: string;
    name: string;
    logo_url?: string | null;
    cover_url?: string | null;
  };
  tournament: components["schemas"]["TournamentSummary"];
  /** A grade tem alguma capa: este tile também leva o espaço. */
  media: boolean;
}) {
  const t = useTranslations("Tournaments");
  const tStatus = useTranslations("TournamentStatus");
  const tAdmission = useTranslations("Admission");
  const locale = useLocale();
  const modeLabel = useModeLabel();
  const mode = modeLabel(tournament.mode, tournament.team_size);
  const spots = t("spots", {
    count: tournament.participants,
    max: tournament.max_participants,
  });

  return (
    <LinkTile
      href={`/${tenant.slug}/${tournament.slug}`}
      title={localizedName(tournament, locale)}
      media={
        media && (
          <TournamentMedia
            cover={tournament.cover_url}
            tenantCover={tenant.cover_url}
            logo={tenant.logo_url}
            community={tenant.name}
          />
        )
      }
      meta={
        <MetaLines
          lines={[
            tournament.starts_at ? (
              <LocalDateTime key="starts" value={tournament.starts_at} />
            ) : (
              t("noDate")
            ),
            `${mode} · ${spots}`,
          ]}
        />
      }
      tags={
        <>
          <StatusTag status={tournament.status}>
            {tStatus(tournament.status)}
          </StatusTag>
          {checkInPending(tournament) ? (
            <StatusTag status="check_in">{t("checkInPending")}</StatusTag>
          ) : (
            tournament.registered && (
              <StatusTag status="registration">{t("registered")}</StatusTag>
            )
          )}
          {tournament.admission === "invite_only" &&
            participantsEditable(tournament.status) && (
              <StatusTag status="neutral">{tAdmission("inviteOnly")}</StatusTag>
            )}
        </>
      }
    />
  );
}
