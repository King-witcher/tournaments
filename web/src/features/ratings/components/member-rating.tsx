import { getTranslations } from "next-intl/server";
import { apiErrorMessage } from "@/shared/api/errors";
import { Column, Grid } from "@/shared/ui/carbon/grid";
import { Link } from "@/shared/ui/link";
import { Notice } from "@/shared/ui/notice";
import { Panel } from "@/shared/ui/panel";
import { Section } from "@/shared/ui/section";
import { RATING_GUIDE_HREF } from "../lib/guide-href";
import { getMemberRating, getRanking } from "../server";
import { AdjustRatingButton } from "./adjust-rating-button";
import { MapRecords } from "./map-records";
import styles from "./member-rating.module.scss";
import { RatedGames } from "./rated-games";
import { RatingAdjustments } from "./rating-adjustments";
import { RatingSummary } from "./rating-summary";

type Props = {
  tenant: string;
  userId: number;
  nickname: string;
  /** Quem vê é organizer ou owner: ajusta e vê a lista de ajustes. */
  staff: boolean;
};

/** O bloco do rating, que recebe o foco depois de um ajuste. */
const RATING_ID = "member-rating";

/**
 * O rating da pessoa na página dela: o resumo (com "Ajustar rating" para a
 * organização) e, ao lado, o desempenho por mapa; embaixo, as últimas partidas que contaram e,
 * para a organização, os ajustes. Fica em `<Suspense>`.
 */
export async function MemberRating({ tenant, userId, nickname, staff }: Props) {
  const [loaded, ranking, t] = await Promise.all([
    getMemberRating(tenant, userId),
    getRanking(tenant),
    getTranslations("Ratings.member"),
  ]);
  if (!loaded.ok) {
    return (
      <Section title={t("title")}>
        <Notice kind="error">{await apiErrorMessage(loaded.code)}</Notice>
      </Section>
    );
  }
  const { rating } = loaded;

  const summary = (
    <div id={RATING_ID} tabIndex={-1}>
      <Panel>
        <Section
          title={t("title")}
          description={
            <Link inline href={RATING_GUIDE_HREF}>
              {t("guide")}
            </Link>
          }
          action={
            staff && (
              <AdjustRatingButton
                tenant={tenant}
                userId={userId}
                nickname={nickname}
                focusId={RATING_ID}
              />
            )
          }
        >
          {rating.standing ? (
            <RatingSummary
              standing={rating.standing}
              total={ranking?.players.length}
              adjusted={rating.adjusted}
              provisionalGames={rating.provisional_games}
            />
          ) : (
            <p>{t("none", { initial: rating.initial })}</p>
          )}
        </Section>
      </Panel>
    </div>
  );

  return (
    <>
      {/* O resumo e o desempenho por mapa lado a lado (do lg para cima): os
          dois respondem "como ela joga" e cabem numa tela; as tabelas vêm
          embaixo, na largura toda. */}
      <Grid>
        <Column sm={4} md={8} lg={rating.by_map.length > 0 ? 6 : 8}>
          {summary}
        </Column>
        {rating.by_map.length > 0 && (
          <Column sm={4} md={8} lg={10} className={styles.maps}>
            <Panel>
              <MapRecords records={rating.by_map} />
            </Panel>
          </Column>
        )}
      </Grid>
      {rating.games.length > 0 && (
        <RatedGames tenant={tenant} games={rating.games} />
      )}
      {rating.adjustments && (
        <RatingAdjustments adjustments={rating.adjustments} />
      )}
    </>
  );
}
