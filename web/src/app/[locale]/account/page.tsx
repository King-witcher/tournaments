import { getTranslations } from "next-intl/server";
import { Suspense } from "react";

import { AccountTab } from "@/features/auth/components/account-tab";
import { AuthError } from "@/features/auth/components/auth-error";
import { DisplayNameSection } from "@/features/auth/components/display-name-section";
import { FormSkeleton } from "@/features/auth/components/form-skeleton";

/** Aba Perfil da conta: o nome exibido. */
export default async function AccountProfilePage({
  searchParams,
}: PageProps<"/[locale]/account">) {
  const t = await getTranslations("Account");
  return (
    <AccountTab>
      {/* Links antigos do Discord voltavam para /account com o erro. */}
      <Suspense fallback={null}>
        <AuthError searchParams={searchParams} />
      </Suspense>
      <Suspense
        fallback={<FormSkeleton heading fields={1} label={t("loading")} />}
      >
        <DisplayNameSection />
      </Suspense>
    </AccountTab>
  );
}
