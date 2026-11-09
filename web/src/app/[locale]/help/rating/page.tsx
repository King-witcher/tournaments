import type { Metadata, ResolvingMetadata } from "next";
import { getTranslations } from "next-intl/server";

import { RatingGuide } from "@/features/ratings/components/rating-guide";
import { RATING_GUIDE_HREF } from "@/features/ratings/lib/guide-href";
import { publicPageMetadata } from "@/shared/lib/link-preview";
import { Page } from "@/shared/ui/page";
import { PageHeader } from "@/shared/ui/page-header";

export async function generateMetadata(
  _: PageProps<"/[locale]/help/rating">,
  parent: ResolvingMetadata,
): Promise<Metadata> {
  const t = await getTranslations("RatingGuide");
  const preview = { title: t("title"), description: t("description") };
  return {
    ...preview,
    ...(await publicPageMetadata(RATING_GUIDE_HREF, preview, parent)),
  };
}

/** O artigo que explica o rating, público e estático. */
export default async function RatingGuidePage() {
  const t = await getTranslations("RatingGuide");
  return (
    <Page>
      <PageHeader title={t("title")} />
      <RatingGuide />
    </Page>
  );
}
