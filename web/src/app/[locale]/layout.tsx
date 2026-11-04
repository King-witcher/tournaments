// Os estilos globais (Carbon e temas) entram antes de qualquer componente,
// para os .module.scss virem depois deles na ordem do CSS.
import "../globals.scss";

import type { Metadata, Viewport } from "next";
import { notFound } from "next/navigation";
import { locale as rootLocale } from "next/root-params";
import { hasLocale, NextIntlClientProvider } from "next-intl";
import { getTranslations } from "next-intl/server";
import { Suspense } from "react";

import { UserMenu } from "@/features/auth/components/user-menu";
import { NotificationBell } from "@/features/notifications/components/notification-bell";
import { PushSession } from "@/features/push/components/push-session";
import { RealtimeProvider } from "@/features/realtime/components/realtime-provider";
import { RealtimeSession } from "@/features/realtime/components/realtime-session";
import { TenantSwitcher } from "@/features/tenants/components/tenant-switcher";
import { routing } from "@/i18n/routing";
import { cn } from "@/shared/lib/cn";
import { plexMono, plexSans } from "@/shared/lib/fonts";
import { SITE_ORIGIN, siteOpenGraph } from "@/shared/lib/link-preview";
import { themeInitScript } from "@/shared/lib/theme";
import { AppContent } from "@/shared/ui/app-content";
import { AppHeader } from "@/shared/ui/app-header";
import { HeaderPlaceholder } from "@/shared/ui/header-placeholder";

export function generateStaticParams() {
  return routing.locales.map((locale) => ({ locale }));
}

/**
 * A cor do Carbon no fundo de cada tema (white e gray 100), na barra do
 * navegador do celular e na faixa da prévia no Discord.
 */
export const viewport: Viewport = {
  themeColor: [
    { media: "(prefers-color-scheme: light)", color: "#ffffff" },
    { media: "(prefers-color-scheme: dark)", color: "#161616" },
  ],
};

export async function generateMetadata(): Promise<Metadata> {
  const [t, tLanding] = await Promise.all([
    getTranslations("Metadata"),
    getTranslations("Landing"),
  ]);
  return {
    metadataBase: SITE_ORIGIN,
    title: t("title"),
    description: t("description"),
    // Só as páginas abertas pedem busca (`publicPageMetadata`): o resto é
    // login, conta ou link com token.
    robots: { index: false, follow: false },
    // A imagem é a de opengraph-image.tsx. O nome já está nela e em
    // `og:site_name`, então o título da prévia não o repete.
    openGraph: {
      ...(await siteOpenGraph()),
      title: tLanding("hero.title"),
      description: tLanding("hero.lead"),
    },
    // Grande no Discord e no X, não a miniatura.
    twitter: { card: "summary_large_image" },
  };
}

export default async function LocaleLayout({
  children,
}: LayoutProps<"/[locale]">) {
  const locale = await rootLocale();
  if (!hasLocale(routing.locales, locale)) notFound();
  const t = await getTranslations("Nav");

  return (
    <html
      lang={locale}
      className={cn(plexSans.variable, plexMono.variable)}
      // O script do tema põe data-theme antes da hidratação.
      suppressHydrationWarning
    >
      <head>
        <script
          // biome-ignore lint/security/noDangerouslySetInnerHtml: constante nossa, sem entrada de usuário
          dangerouslySetInnerHTML={{ __html: themeInitScript }}
        />
      </head>
      <body>
        <NextIntlClientProvider>
          {/* As páginas dizem o que mostram (LiveUpdates); a conexão abre
              só para quem está logado (RealtimeSession), e a inscrição de
              Web Push acompanha o idioma e a conta (PushSession). */}
          <RealtimeProvider>
            <AppHeader name={t("brand")} skipLabel={t("skipToContent")}>
              {/* O lugar de cada ação enquanto ela chega: a barra não pula.
                  As que só existem com sessão (comunidades, sininho) ficam à
                  esquerda: sem sessão, somem sem mexer nas outras. */}
              <Suspense fallback={<HeaderPlaceholder />}>
                <TenantSwitcher />
              </Suspense>
              <Suspense fallback={<HeaderPlaceholder />}>
                <NotificationBell />
              </Suspense>
              <Suspense fallback={<HeaderPlaceholder />}>
                <UserMenu />
              </Suspense>
            </AppHeader>
            <AppContent>{children}</AppContent>
            <Suspense fallback={null}>
              <RealtimeSession />
            </Suspense>
            <Suspense fallback={null}>
              <PushSession />
            </Suspense>
          </RealtimeProvider>
        </NextIntlClientProvider>
      </body>
    </html>
  );
}
