import { getTranslations } from "next-intl/server";

import { InviteSkeleton } from "@/features/invites/components/invite-skeleton";
import { Page } from "@/shared/ui/page";

/** O convite carregando: o título e o tile com o que ele é e o botão. */
export default async function Loading() {
  const t = await getTranslations("Invite");
  return (
    <Page>
      <InviteSkeleton label={t("loading")} />
    </Page>
  );
}
