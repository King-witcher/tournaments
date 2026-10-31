import { getTranslations } from "next-intl/server";
import { type ReactNode, Suspense } from "react";

import { RequireLogin } from "@/features/auth/components/require-login";
import { Page } from "@/shared/ui/page";
import { PageHeader } from "@/shared/ui/page-header";
import { PageTabs } from "@/shared/ui/page-tabs";
import { FormSkeleton, TabLoading, TabsSkeleton } from "@/shared/ui/skeleton";

/**
 * A conta: o título e as abas Perfil, Logins, Sessões e Notificações, cada uma uma
 * sub-rota (as que buscam dado, com o próprio `loading.tsx`). Sem sessão, no
 * lugar das abas, o convite a entrar (que volta para cá).
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
    <Page>
      <PageHeader title={t("title")} />
      <Suspense
        fallback={
          <>
            <TabsSkeleton />
            <TabLoading label={t("loading")}>
              <FormSkeleton heading fields={1} label={false} />
            </TabLoading>
          </>
        }
      >
        <RequireLogin message={t("loginRequired")}>
          <PageTabs
            label={t("sections")}
            tabs={[
              { href: "/account", label: t("tabs.profile") },
              { href: "/account/logins", label: t("tabs.logins") },
              { href: "/account/sessions", label: t("tabs.sessions") },
              {
                href: "/account/notifications",
                label: t("tabs.notifications"),
              },
            ]}
          >
            {children}
          </PageTabs>
        </RequireLogin>
      </Suspense>
    </Page>
  );
}
