import { useLocale } from "next-intl";
import { Suspense } from "react";

import { MySituationLine } from "@/features/participants/components/my-situation-line";
import type { components } from "@/shared/api/schema";
import { SkeletonText } from "@/shared/ui/carbon/skeleton-text";
import { PageHeader } from "@/shared/ui/page-header";
import { TOURNAMENT_TITLE_ID } from "../lib/focus";
import { localizedName } from "../lib/localize";
import { hasStarted } from "../lib/status";
import { TournamentActions } from "./tournament-actions";
import { TournamentOverflow } from "./tournament-overflow";
import { TournamentSectionTitle } from "./tournament-section-title";

type Schemas = components["schemas"];

type Props = {
  tenant: string;
  /** A comunidade, com o papel de quem vê. */
  me: Schemas["MyTenant"];
  tournament: Schemas["Tournament"];
  /** Referência para saber se a abertura do check-in já passou. */
  now: Date;
};

/**
 * O cabeçalho das seções do torneio que não são a Visão geral: o título é a
 * seção, e o torneio fica no menu lateral (abaixo do `lg`, onde o menu fica
 * atrás do botão, é pela trilha que se sabe o torneio). Sem a etiqueta do estado,
 * a capa e o metadado, que o menu do torneio e a Visão geral já mostram: a
 * seção começa logo. As mesmas ações do cabeçalho completo, para o que falta
 * fazer não sumir ao trocar de seção. Para quem joga, depois do início, a
 * situação numa linha, com a próxima série.
 */
export function TournamentSectionHeader({
  tenant,
  me,
  tournament,
  now,
}: Props) {
  const locale = useLocale();
  const organizer = me.role !== "player";

  return (
    <PageHeader
      breadcrumbs={[
        {
          href: `/${tenant}/${tournament.slug}`,
          label: localizedName(tournament, locale),
        },
      ]}
      inSideNav
      title={
        <span id={TOURNAMENT_TITLE_ID} tabIndex={-1}>
          <TournamentSectionTitle />
        </span>
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
          owner={me.role === "owner"}
        />
      }
    >
      {tournament.registered && hasStarted(tournament.status) && (
        <Suspense fallback={<SkeletonText width="40%" />}>
          <MySituationLine tenant={tenant} tournament={tournament} />
        </Suspense>
      )}
    </PageHeader>
  );
}
