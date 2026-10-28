// Os estilos globais (Carbon e temas) entram antes de qualquer componente,
// para os .module.scss virem depois deles na ordem do CSS.
import "../globals.scss";

import type { Metadata } from "next";
import { IBM_Plex_Mono, IBM_Plex_Sans } from "next/font/google";
import { notFound } from "next/navigation";
import { locale as rootLocale } from "next/root-params";
import { hasLocale, NextIntlClientProvider } from "next-intl";
import { getTranslations } from "next-intl/server";
import { Suspense } from "react";

import { UserMenu } from "@/features/auth/components/user-menu";
import { PushSession } from "@/features/push/components/push-session";
import { RealtimeProvider } from "@/features/realtime/components/realtime-provider";
import { RealtimeSession } from "@/features/realtime/components/realtime-session";
import { routing } from "@/i18n/routing";
import { cn } from "@/shared/lib/cn";
import { themeInitScript } from "@/shared/lib/theme";
import { AppContent } from "@/shared/ui/app-content";
import { AppHeader } from "@/shared/ui/app-header";
import { HeaderPlaceholder } from "@/shared/ui/header-placeholder";
import { LocaleSwitcher } from "@/shared/ui/locale-switcher";

// IBM Plex, a fonte do Carbon, servida pelo next/font (sem CDN). As
// variáveis são lidas pelas famílias do Carbon em globals.scss.
const plexSans = IBM_Plex_Sans({
  variable: "--font-plex-sans",
  subsets: ["latin"],
});

const plexMono = IBM_Plex_Mono({
  variable: "--font-plex-mono",
  subsets: ["latin"],
  weight: ["400", "600"],
});

export function generateStaticParams() {
  return routing.locales.map((locale) => ({ locale }));
}

export async function generateMetadata(): Promise<Metadata> {
  const t = await getTranslations("Metadata");
  return { title: t("title"), description: t("description") };
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
              {/* O lugar de cada ação enquanto ela chega: a barra não pula. */}
              <Suspense fallback={<HeaderPlaceholder />}>
                <LocaleSwitcher />
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
