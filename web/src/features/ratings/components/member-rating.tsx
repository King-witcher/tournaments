import { useTranslations } from "next-intl";

import type { components } from "@/shared/api/schema";
import { Link } from "@/shared/ui/link";
import { Panel } from "@/shared/ui/panel";
import { Section } from "@/shared/ui/section";
import { RATING_GUIDE_HREF } from "../lib/guide-href";
import { AdjustRatingButton } from "./adjust-rating-button";
import { RatingSummary } from "./rating-summary";

type Props = {
  tenant: string;
  rating: components["schemas"]["MemberRating"];
  nickname: string;
  /** Quem vê é organizer ou owner: ajusta. */
  staff: boolean;
};

/** O bloco do rating, que recebe o foco depois de um ajuste. */
const RATING_ID = "member-rating";

/**
 * "Rating" na Visão geral da pessoa: o resumo, com o link do artigo e, para a
 * organização, "Ajustar rating"; sem mapa nem ajuste, o rating inicial.
 */
export function MemberRating({ tenant, rating, nickname, staff }: Props) {
  const t = useTranslations("Ratings.member");
  return (
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
                userId={rating.user_id}
                nickname={nickname}
                focusId={RATING_ID}
              />
            )
          }
        >
          {rating.standing ? (
            <RatingSummary
              standing={rating.standing}
              total={rating.ranked}
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
}
