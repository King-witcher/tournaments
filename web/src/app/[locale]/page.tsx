import { getTranslations } from "next-intl/server";
import { Suspense } from "react";

import { AnonymousIntro } from "@/features/auth/components/anonymous-intro";
import { AuthError } from "@/features/auth/components/auth-error";
import { ApiStatus } from "@/features/health/components/api-status";
import { MyTenants } from "@/features/tenants/components/my-tenants";
import { MyTournaments } from "@/features/tournaments/components/my-tournaments";
import { Page } from "@/shared/ui/page";
import { LoadingSection } from "@/shared/ui/skeleton";

export default async function Home({ searchParams }: PageProps<"/[locale]">) {
  const t = await getTranslations("Home");

  return (
    <Page spacing="loose">
      <Suspense fallback={null}>
        <AuthError searchParams={searchParams} />
      </Suspense>
      <header className="flex flex-col gap-2">
        <h1 className="text-3xl font-semibold">{t("title")}</h1>
        <p className="text-muted">{t("subtitle")}</p>
      </header>
      <Suspense fallback={null}>
        <ApiStatus />
      </Suspense>
      <Suspense fallback={null}>
        <AnonymousIntro />
      </Suspense>
      <Suspense fallback={<LoadingSection rows={2} />}>
        <MyTournaments />
      </Suspense>
      <Suspense fallback={<LoadingSection rows={2} />}>
        <MyTenants />
      </Suspense>
    </Page>
  );
}
