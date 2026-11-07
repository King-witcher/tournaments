import { getTranslations } from "next-intl/server";
import { type ReactNode, Suspense } from "react";

import { AccountGate } from "@/features/auth/components/account-gate";
import { AccountSideNav } from "@/features/auth/components/account-side-nav";
import { AccountSideNavSwitch } from "@/features/auth/components/account-side-nav-switch";
import { AppSideNav } from "@/shared/ui/app-side-nav";
import { Page } from "@/shared/ui/page";
import { PageHeaderSkeleton, SectionSkeleton } from "@/shared/ui/skeleton";

/**
 * A conta: o menu lateral dela (Perfil, Logins, Sessões e Notificações, cada
 * uma uma sub-rota, as que buscam dado com o próprio `loading.tsx`), que não
 * remonta ao navegar, e a página larga. Cada seção desenha o próprio
 * cabeçalho. Sem sessão, no lugar delas, o convite a entrar (que volta para
 * cá). Enquanto a sessão carrega, o menu vazio já ocupa o lugar, e a página
 * não pula.
 */
// As props tipadas à mão: o `LayoutProps` desta rota só existe depois que o
// Next regenera os tipos das rotas (`next dev`/`next build`).
export default async function AccountLayout({
  children,
}: {
  children: ReactNode;
}) {
  const t = await getTranslations("Account");
  return (
    <>
      <Suspense
        fallback={
          <AccountSideNavSwitch>
            <AppSideNav label={t("nav")}>{null}</AppSideNav>
          </AccountSideNavSwitch>
        }
      >
        <AccountSideNav />
      </Suspense>
      <Page width="wide">
        <Suspense
          fallback={
            <>
              <PageHeaderSkeleton breadcrumb={false} label={t("loading")} />
              <SectionSkeleton rows={2} />
            </>
          }
        >
          <AccountGate>{children}</AccountGate>
        </Suspense>
      </Page>
    </>
  );
}
