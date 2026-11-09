import { getTranslations } from "next-intl/server";
import { getCurrentUser } from "@/features/auth/server";
import { MemberLink } from "@/features/members/components/member-link";
import { Link } from "@/shared/ui/link";
import { Panel } from "@/shared/ui/panel";
import { Section } from "@/shared/ui/section";
import { StructuredList } from "@/shared/ui/structured-list";
import { getRanking } from "../server";
import styles from "./ranking-top.module.scss";
import { RatingValue } from "./rating-value";

const TOP = 5;

/**
 * O topo do ranking na coluna estreita do início da comunidade: os cinco
 * primeiros (o apelido abre a página da pessoa), a posição de quem vê quando
 * ela não está entre eles e o link para o ranking. Sem ninguém classificado
 * (ou sem acesso), nada. Fica em `<Suspense>`.
 */
export async function RankingTop({ slug }: { slug: string }) {
  const [ranking, me, t] = await Promise.all([
    getRanking(slug),
    getCurrentUser(),
    getTranslations("Ratings"),
  ]);
  if (!ranking?.players.length) return null;
  const top = ranking.players.slice(0, TOP);
  const mine = ranking.players.find((p) => p.user_id === me?.id);
  const outside = mine && !top.includes(mine);

  return (
    <Panel>
      <Section title={t("top.title")}>
        <StructuredList
          label={t("ranking.label")}
          condensed
          headers={[
            t("ranking.position"),
            t("ranking.member"),
            t("ranking.rating"),
          ]}
          rows={top.map((p) => ({
            key: String(p.user_id),
            cells: [
              t("member.positionShort", { position: p.position }),
              <MemberLink key="member" tenant={slug} userId={p.user_id}>
                {p.nickname}
              </MemberLink>,
              <RatingValue
                key="rating"
                rating={p.rating}
                provisional={p.provisional}
              />,
            ],
          }))}
        />
        {outside && (
          <p className={styles.mine}>
            {t("top.mine", {
              position: t("member.positionShort", { position: mine.position }),
              rating: mine.rating,
            })}
          </p>
        )}
        <Link href={`/${slug}/ranking`}>{t("top.all")}</Link>
      </Section>
    </Panel>
  );
}
