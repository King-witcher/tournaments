import { getTranslations } from "next-intl/server";

import { HomeColumns } from "@/features/auth/components/home-columns";
import { JoinWithCodeButton } from "@/features/invites/components/join-with-code-button";
import { VetoRoomEntry } from "@/features/veto-rooms/components/veto-room-entry";
import { Page } from "@/shared/ui/page";
import { PageHeader } from "@/shared/ui/page-header";
import { SectionSkeleton, TabLoading } from "@/shared/ui/skeleton";

/**
 * O início carregando, na forma dele: o cabeçalho já pronto (o título e a
 * ação não dependem de dado) e, nas colunas do início, os tiles da coluna
 * principal e, ao lado (no celular, embaixo), as comunidades.
 */
export default async function Loading() {
  const t = await getTranslations("Home");
  return (
    <Page width="wide">
      <PageHeader
        title={t("title")}
        actions={
          <>
            <JoinWithCodeButton />
            <VetoRoomEntry />
          </>
        }
      />
      <TabLoading label={t("loading")}>
        <HomeColumns
          lead={<SectionSkeleton tiles={3} />}
          aside={<SectionSkeleton tiles={2} />}
        />
      </TabLoading>
    </Page>
  );
}
