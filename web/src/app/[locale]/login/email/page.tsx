import { getTranslations } from "next-intl/server";
import { Suspense } from "react";

import { EmailLoginView } from "@/features/auth/components/email-login-view";
import { Page } from "@/shared/ui/page";
import { PageHeader } from "@/shared/ui/page-header";
import { FormSkeleton } from "@/shared/ui/skeleton";

/** Entrar por email: pede o link e o código, e recebe o código. */
export default async function EmailLoginPage({
  searchParams,
}: {
  searchParams: Promise<Record<string, string | string[] | undefined>>;
}) {
  const t = await getTranslations("Login");
  return (
    <Page>
      <PageHeader
        breadcrumbs={[{ href: "/login", label: t("title") }]}
        title={t("email.title")}
        meta={t("email.text")}
      />
      <Suspense fallback={<FormSkeleton tile />}>
        <EmailLoginView searchParams={searchParams} />
      </Suspense>
    </Page>
  );
}
