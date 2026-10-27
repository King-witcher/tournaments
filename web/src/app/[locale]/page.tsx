import { getTranslations } from "next-intl/server";
import { Suspense } from "react";

import { ApiStatus } from "@/components/api-status";
import { AuthError } from "@/components/auth-error";
import { MyTenants } from "@/components/my-tenants";

export default async function Home({ searchParams }: PageProps<"/[locale]">) {
  const t = await getTranslations("Home");

  return (
    <main className="mx-auto flex w-full max-w-2xl flex-1 flex-col gap-4 px-4 py-16">
      <Suspense fallback={null}>
        <AuthError searchParams={searchParams} />
      </Suspense>
      <h1 className="text-3xl font-semibold">{t("title")}</h1>
      <p>{t("subtitle")}</p>
      <Suspense fallback={<p>{t("status.checking")}</p>}>
        <ApiStatus />
      </Suspense>
      <Suspense fallback={null}>
        <MyTenants />
      </Suspense>
    </main>
  );
}
