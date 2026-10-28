import { getTranslations } from "next-intl/server";
import { Suspense } from "react";
import { LoginView } from "@/features/auth/components/login-view";
import { Page } from "@/shared/ui/page";
import { PageHeader } from "@/shared/ui/page-header";
import { FormSkeleton } from "@/shared/ui/skeleton";

export default async function LoginPage({
  searchParams,
}: PageProps<"/[locale]/login">) {
  const t = await getTranslations("Login");
  return (
    <Page>
      <PageHeader title={t("title")} />
      <Suspense fallback={<FormSkeleton tile />}>
        <LoginView searchParams={searchParams} />
      </Suspense>
    </Page>
  );
}
