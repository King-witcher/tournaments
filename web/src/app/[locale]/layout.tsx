import type { Metadata } from "next";
import { Geist, Geist_Mono } from "next/font/google";
import { notFound } from "next/navigation";
import { locale as rootLocale } from "next/root-params";
import { hasLocale, NextIntlClientProvider } from "next-intl";
import { getTranslations } from "next-intl/server";

import { Suspense } from "react";

import { UserMenu } from "@/features/auth/components/user-menu";
import { Link } from "@/i18n/navigation";
import { routing } from "@/i18n/routing";
import { LocaleSwitcher } from "@/shared/ui/locale-switcher";
import "../globals.css";

const geistSans = Geist({
  variable: "--font-geist-sans",
  subsets: ["latin"],
});

const geistMono = Geist_Mono({
  variable: "--font-geist-mono",
  subsets: ["latin"],
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
      className={`${geistSans.variable} ${geistMono.variable} h-full antialiased`}
    >
      <body className="min-h-full flex flex-col">
        <NextIntlClientProvider>
          <header className="flex items-center justify-between gap-4 border-b border-black/10 px-4 py-3 dark:border-white/10">
            <Link
              href="/"
              className="inline-flex min-h-11 items-center sm:min-h-0 text-sm font-medium underline underline-offset-4 hover:decoration-2"
            >
              {t("home")}
            </Link>
            <div className="flex items-center gap-4">
              <Suspense fallback={null}>
                <LocaleSwitcher />
              </Suspense>
              <Suspense fallback={null}>
                <UserMenu />
              </Suspense>
            </div>
          </header>
          {children}
        </NextIntlClientProvider>
      </body>
    </html>
  );
}
