import { getTranslations } from "next-intl/server";
import { Suspense } from "react";

import { AccountColumns } from "@/features/auth/components/account-columns";
import { AccountDeletionCancelled } from "@/features/auth/components/account-deletion-cancelled";
import { AuthError } from "@/features/auth/components/auth-error";
import { DeleteAccountSection } from "@/features/auth/components/delete-account-section";
import { DisplayNameSection } from "@/features/auth/components/display-name-section";
import { AvatarSection } from "@/features/images/components/avatar-section";
import { PageHeader } from "@/shared/ui/page-header";
import { FormSkeleton, SectionSkeleton } from "@/shared/ui/skeleton";

/**
 * O Perfil, o início da conta: o nome exibido à esquerda, a foto à direita e,
 * no fim, a zona de perigo com excluir a conta (`AccountColumns`).
 */
export default async function AccountProfilePage({
  searchParams,
}: PageProps<"/[locale]/account">) {
  const t = await getTranslations("Account");
  return (
    <>
      <PageHeader title={t("tabs.profile")} />
      {/* Links antigos do Discord voltavam para /account com o erro. */}
      <Suspense fallback={null}>
        <AuthError searchParams={searchParams} />
      </Suspense>
      <Suspense fallback={null}>
        <AccountDeletionCancelled searchParams={searchParams} />
      </Suspense>
      <AccountColumns
        start={
          <Suspense
            fallback={<FormSkeleton heading fields={1} label={t("loading")} />}
          >
            <DisplayNameSection />
          </Suspense>
        }
        end={
          <Suspense fallback={<SectionSkeleton rows={2} />}>
            <AvatarSection />
          </Suspense>
        }
        danger={
          <Suspense fallback={<SectionSkeleton rows={2} />}>
            <DeleteAccountSection />
          </Suspense>
        }
      />
    </>
  );
}
