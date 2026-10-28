import { getTranslations } from "next-intl/server";
import { Suspense } from "react";

import { InviteSkeleton } from "@/features/invites/components/invite-skeleton";
import { InviteView } from "@/features/invites/components/invite-view";
import { Page } from "@/shared/ui/page";

export default async function InvitePage({
  params,
}: PageProps<"/[locale]/invite/[token]">) {
  const t = await getTranslations("Invite");
  return (
    <Page>
      <Suspense fallback={<InviteSkeleton label={t("loading")} />}>
        {params.then(({ token }) => (
          <InviteView token={token} />
        ))}
      </Suspense>
    </Page>
  );
}
