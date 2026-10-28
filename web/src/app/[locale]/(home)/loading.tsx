import { getTranslations } from "next-intl/server";

import { Page } from "@/shared/ui/page";
import { PageHeaderSkeleton, SectionSkeleton } from "@/shared/ui/skeleton";

/** O início carregando, na forma dele: título e frase, seus torneios e suas comunidades. */
export default async function Loading() {
  const t = await getTranslations("Home");
  return (
    <Page width="wide">
      <PageHeaderSkeleton breadcrumb={false} meta label={t("loading")} />
      <SectionSkeleton tiles={3} />
      <SectionSkeleton tiles={2} />
    </Page>
  );
}
