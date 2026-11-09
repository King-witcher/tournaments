import { getTranslations } from "next-intl/server";
import { AccountColumns } from "@/features/auth/components/account-columns";
import { Panel } from "@/shared/ui/panel";
import { PageHeaderSkeleton, SectionSkeleton } from "@/shared/ui/skeleton";

/** As Notificações carregando: o cabeçalho e as duas seções lado a lado. */
export default async function Loading() {
  const t = await getTranslations("Account");
  return (
    <>
      <PageHeaderSkeleton label={t("loading")} />
      <AccountColumns
        start={
          <Panel>
            <SectionSkeleton rows={2} />
          </Panel>
        }
        end={
          <Panel>
            <SectionSkeleton rows={2} />
          </Panel>
        }
      />
    </>
  );
}
