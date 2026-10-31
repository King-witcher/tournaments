import { getTranslations } from "next-intl/server";

import { apiErrorMessage } from "@/shared/api/errors";
import { Notice } from "@/shared/ui/notice";
import { Section } from "@/shared/ui/section";
import { getMemberRating, getRanking } from "../server";
import { AdjustRatingButton } from "./adjust-rating-button";
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
 * organização), os últimos mapas que contaram e, para a organização, os
 * ajustes. Fica em `<Suspense>`.
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

  return (
    <>
      <div id={RATING_ID} tabIndex={-1}>
        <Section
          title={t("title")}
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
      </div>
      {rating.games.length > 0 && (
        <RatedGames tenant={tenant} games={rating.games} />
      )}
      {rating.adjustments && (
        <RatingAdjustments adjustments={rating.adjustments} />
      )}
    </>
  );
}
