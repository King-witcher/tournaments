import { getTranslations } from "next-intl/server";
import { Suspense } from "react";
import { TenantView } from "@/components/tenant-view";

export default async function TenantPage({
  params,
}: PageProps<"/[locale]/[tenant]">) {
  const t = await getTranslations("Tenant");
  return (
    <main className="mx-auto flex w-full max-w-2xl flex-1 flex-col gap-8 px-4 py-16">
      <Suspense fallback={<p>{t("loading")}</p>}>
        {params.then(({ tenant }) => (
          <TenantView slug={tenant} />
        ))}
      </Suspense>
    </main>
  );
}
