import { MapRecords } from "@/features/ratings/components/map-records";
import { MemberRating } from "@/features/ratings/components/member-rating";
import { RatingAdjustments } from "@/features/ratings/components/rating-adjustments";
import { RatingHistoryChart } from "@/features/ratings/components/rating-history-chart";
import { getMemberRating } from "@/features/ratings/server";
import { getTenant } from "@/features/tenants/server";
import { apiErrorMessage } from "@/shared/api/errors";
import { Column, Grid } from "@/shared/ui/carbon/grid";
import { Stack } from "@/shared/ui/carbon/stack";
import { Notice } from "@/shared/ui/notice";
import { Panel } from "@/shared/ui/panel";
import { getMember, getMemberCareer, parseUserId } from "../server";
import { MemberCareer } from "./member-career";
import styles from "./member-overview.module.scss";
import { RecentSeries } from "./recent-series";

type Props = {
  tenant: string;
  /** O `user_id` do endereço, ainda como texto. */
  user: string;
};

/**
 * A Visão geral da pessoa, do que resume ao detalhe: Rating e Carreira lado
 * a lado (do `lg` para cima), a evolução do rating, o desempenho por mapa,
 * as últimas séries e, só para a organização, os ajustes manuais. Cada bloco
 * sem dado some (o resumo e a carreira dizem o estado). Sem o membro, nada:
 * a casca já mostra o erro. As buscas saem juntas, com o id do endereço
 * (o membro já vem do cache da casca). Fica em `<Suspense>`.
 */
export async function MemberOverview({ tenant, user }: Props) {
  const userId = parseUserId(user);
  if (userId === undefined) return null;
  const [member, loaded, rating, career] = await Promise.all([
    getMember(tenant, user),
    getTenant(tenant),
    getMemberRating(tenant, userId),
    getMemberCareer(tenant, userId),
  ]);
  if (!member.ok || loaded.kind !== "ok") return null;
  const { nickname } = member.data;
  if (!rating.ok || !career.ok) {
    const code = rating.ok
      ? career.ok
        ? "internal"
        : career.code
      : rating.code;
    return (
      <Panel>
        <Notice kind="error">{await apiErrorMessage(code)}</Notice>
      </Panel>
    );
  }
  const staff = loaded.tenant.role !== "player";
  const { history, by_map: byMap, adjustments } = rating.rating;

  return (
    <Stack gap={7}>
      <Grid>
        <Column sm={4} md={8} lg={8}>
          <MemberRating
            tenant={tenant}
            rating={rating.rating}
            nickname={nickname}
            staff={staff}
          />
        </Column>
        <Column sm={4} md={8} lg={8} className={styles.career}>
          <MemberCareer career={career.data} />
        </Column>
      </Grid>
      {history.length > 0 && <RatingHistoryChart history={history} />}
      {byMap.length > 0 && (
        <Panel>
          <MapRecords records={byMap} />
        </Panel>
      )}
      {career.data.recent_series.length > 0 && (
        <RecentSeries
          tenant={tenant}
          base={`/${tenant}/members/${userId}`}
          series={career.data.recent_series}
        />
      )}
      {adjustments && <RatingAdjustments adjustments={adjustments} />}
    </Stack>
  );
}
