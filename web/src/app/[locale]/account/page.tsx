import { Stack } from "@carbon/react";
import { getTranslations } from "next-intl/server";
import { Suspense } from "react";

import { AccountDeletionCancelled } from "@/features/auth/components/account-deletion-cancelled";
import { AuthError } from "@/features/auth/components/auth-error";
import { DeleteAccountSection } from "@/features/auth/components/delete-account-section";
import { DisplayNameSection } from "@/features/auth/components/display-name-section";
import { FormSkeleton, SectionSkeleton } from "@/shared/ui/skeleton";

/** Aba Perfil da conta: o nome exibido e, no fim, excluir a conta. */
export default async function AccountProfilePage({
  searchParams,
}: PageProps<"/[locale]/account">) {
  const t = await getTranslations("Account");
  return (
    <Stack gap={7}>
      {/* Links antigos do Discord voltavam para /account com o erro. */}
      <Suspense fallback={null}>
        <AuthError searchParams={searchParams} />
      </Suspense>
      <Suspense fallback={null}>
        <AccountDeletionCancelled searchParams={searchParams} />
      </Suspense>
      <Suspense
        fallback={<FormSkeleton heading fields={1} label={t("loading")} />}
      >
        <DisplayNameSection />
      </Suspense>
      <Suspense fallback={<SectionSkeleton rows={2} />}>
        <DeleteAccountSection />
      </Suspense>
    </Stack>
  );
}
