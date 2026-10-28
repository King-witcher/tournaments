import { getTranslations } from "next-intl/server";
import { Suspense } from "react";

import { SignupView } from "@/features/auth/components/signup-view";
import { Page } from "@/shared/ui/page";
import { LoadingSection } from "@/shared/ui/skeleton";

export default async function SignupPage({
  searchParams,
}: PageProps<"/[locale]/signup">) {
  const t = await getTranslations("Signup");
  return (
    <Page>
      <h1 className="text-3xl font-semibold">{t("title")}</h1>
      <Suspense fallback={<LoadingSection rows={2} />}>
        <SignupView searchParams={searchParams} />
      </Suspense>
    </Page>
  );
}
