import { getTranslations } from "next-intl/server";

import { Page } from "@/shared/ui/page";
import { PageColumns } from "@/shared/ui/page-columns";
import { PageHeaderSkeleton, SectionSkeleton } from "@/shared/ui/skeleton";

/** O início carregando, na forma dele: título, seus torneios e, ao lado, suas comunidades. */
export default async function Loading() {
  const t = await getTranslations("Home");
  return (
    <Page width="wide">
      <PageHeaderSkeleton breadcrumb={false} label={t("loading")} />
      <PageColumns
        main={<SectionSkeleton tiles={3} />}
        aside={<SectionSkeleton tiles={2} />}
      />
    </Page>
  );
}
