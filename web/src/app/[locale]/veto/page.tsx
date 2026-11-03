import type { Metadata } from "next";
import { getTranslations } from "next-intl/server";
import { Suspense } from "react";

import { NewVetoRoom } from "@/features/veto-rooms/components/new-veto-room";
import { Page } from "@/shared/ui/page";
import { PageHeader } from "@/shared/ui/page-header";
import { FormSkeleton } from "@/shared/ui/skeleton";

export async function generateMetadata(): Promise<Metadata> {
  const t = await getTranslations("VetoRoom");
  return { title: t("title"), description: t("description") };
}

export default async function NewVetoRoomPage() {
  const [t, tNav] = await Promise.all([
    getTranslations("VetoRoom"),
    getTranslations("Nav"),
  ]);
  return (
    <Page>
      <PageHeader
        breadcrumbs={[{ href: "/", label: tNav("home") }]}
        title={t("title")}
        meta={t("description")}
      />
      <Suspense fallback={<FormSkeleton label={false} />}>
        <NewVetoRoom />
      </Suspense>
    </Page>
  );
}
