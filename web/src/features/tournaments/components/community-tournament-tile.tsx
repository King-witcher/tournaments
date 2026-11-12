import { useLocale, useTranslations } from "next-intl";

import type { components } from "@/shared/api/schema";
import { LinkTile } from "@/shared/ui/link-tile";
import { LocalDateTime } from "@/shared/ui/local-date-time";
import { MetaLines } from "@/shared/ui/meta-lines";
import { StatusTag } from "@/shared/ui/status-tag";
import { useModeLabel } from "../hooks/use-mode-label";
import { localizedName } from "../lib/localize";
import { participantsEditable } from "../lib/status";
import { TournamentMedia } from "./tournament-media";

/**
 * Um torneio ativo na aba Torneios da comunidade: com `media`, a capa no topo
 * (ou o logo da comunidade no lugar dela), o nome, o início, a modalidade e as
 * vagas e, numa linha de texto, o estado, se quem vê está inscrito e "Só por
 * link". Etiqueta só no rascunho, que ainda não é público.
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
  const draft = tournament.status === "draft";
  // O estado e o que é de quem vê, em texto: o caminho esperado não chama
  // atenção, e o check-in que falta já é um aviso no topo da página.
  const facts = [
    !draft && tStatus(tournament.status),
    tournament.registered && t("registered"),
    tournament.admission === "invite_only" &&
      participantsEditable(tournament.status) &&
      tAdmission("inviteOnly"),
  ].filter(Boolean);

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
            facts.join(" · "),
          ]}
        />
      }
      tags={draft && <StatusTag status="neutral">{tStatus("draft")}</StatusTag>}
    />
  );
}
