import { useLocale, useTranslations } from "next-intl";

import type { components } from "@/shared/api/schema";
import { LocalDateTime } from "@/shared/ui/local-date-time";
import { PageHeader } from "@/shared/ui/page-header";
import { StatusTag } from "@/shared/ui/status-tag";
import { TOURNAMENT_TITLE_ID } from "../lib/focus";
import { formatKind } from "../lib/format";
import { localizedName } from "../lib/localize";
import { hasStarted } from "../lib/status";
import { TournamentActions } from "./tournament-actions";
import { TournamentOverflow } from "./tournament-overflow";

type Schemas = components["schemas"];

type Props = {
  tenant: string;
  /** A comunidade, com o papel de quem vê. */
  me: Schemas["MyTenant"];
  tournament: Schemas["Tournament"];
  /** Referência para saber se a data de início já passou. */
  now: Date;
};

/**
 * O topo da página do torneio: a trilha até a comunidade, o nome no idioma
 * de quem vê, o estado (e "Privado"), a linha formato · melhor de · início ·
 * inscritos e as ações de quem vê, com o raro e o destrutivo no ⋯.
 */
export function TournamentHeader({ tenant, me, tournament, now }: Props) {
  const t = useTranslations("Tournament");
  const tNav = useTranslations("Nav");
  const tStatus = useTranslations("TournamentStatus");
  const tVisibility = useTranslations("Visibility");
  const locale = useLocale();
  const organizer = me.role !== "player";
  const [stage] = tournament.stages;
  const kind = formatKind(stage);
  const startsAt = tournament.starts_at;
  // Data que já passou (começou, ou a organização ainda não iniciou): "início
  // marcado para", não "começa", que daria a entender que ainda vai começar.
  const past =
    hasStarted(tournament.status) ||
    (startsAt != null && new Date(startsAt) <= now);

  return (
    <PageHeader
      breadcrumbs={[
        { href: "/", label: tNav("home") },
        { href: `/${tenant}`, label: me.name },
      ]}
      // tabIndex -1: recebe o foco depois de publicar, iniciar, desfazer…,
      // quando o botão usado some no refresh.
      title={
        <span id={TOURNAMENT_TITLE_ID} tabIndex={-1}>
          {localizedName(tournament, locale)}
        </span>
      }
      tags={
        <>
          <StatusTag status={tournament.status}>
            {tStatus(tournament.status)}
          </StatusTag>
          {tournament.visibility === "private" && (
            <StatusTag status="neutral">{tVisibility("private")}</StatusTag>
          )}
        </>
      }
      meta={
        <>
          {t(`setup.bracket.${kind}`)}
          {" · "}
          {t("meta.bestOf", { bestOf: stage.best_of })}
          {" · "}
          {startsAt
            ? t.rich(past ? "startedAt" : "startsAt", {
                date: () => <LocalDateTime value={startsAt} />,
              })
            : t("noStartDate")}
          {" · "}
          {t("meta.participants", {
            count: tournament.participants,
            max: tournament.max_participants,
          })}
        </>
      }
      actions={
        <TournamentActions
          tenant={tenant}
          tournament={tournament}
          organizer={organizer}
          now={now}
        />
      }
      overflow={
        <TournamentOverflow
          tenant={tenant}
          tournament={tournament}
          organizer={organizer}
        />
      }
    />
  );
}
