import { getTranslations } from "next-intl/server";
import { Suspense } from "react";

import { FormSkeleton } from "@/features/auth/components/form-skeleton";
import { SignupView } from "@/features/auth/components/signup-view";
import { Page } from "@/shared/ui/page";
import { PageHeader } from "@/shared/ui/page-header";

export default async function SignupPage({
  searchParams,
}: PageProps<"/[locale]/signup">) {
  const t = await getTranslations("Signup");
  return (
    <Page>
      <PageHeader title={t("title")} />
      <Suspense fallback={<FormSkeleton fields={3} text tile />}>
        <SignupView searchParams={searchParams} />
      </Suspense>
    </Page>
  );
}
