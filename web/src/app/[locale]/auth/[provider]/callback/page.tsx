import { InlineLoading } from "@carbon/react";
import type { Metadata } from "next";
import { notFound } from "next/navigation";
import { getTranslations } from "next-intl/server";
import { Suspense } from "react";

import { OAuthCallback } from "@/features/auth/components/oauth-callback";
import { isProvider } from "@/features/auth/lib/providers";
import { Page } from "@/shared/ui/page";

/** O code está na URL: nada que a página carregar o recebe no Referer. */
export const metadata: Metadata = { referrer: "no-referrer" };

/**
 * A volta do OAuth do provedor. O redirect registrado nele não tem idioma
 * (`/auth/{provedor}/callback`): o proxy põe, com a query.
 */
export default async function OAuthCallbackPage({
  params,
}: PageProps<"/[locale]/auth/[provider]/callback">) {
  const t = await getTranslations("Auth");
  return (
    <Page>
      <Suspense fallback={<InlineLoading description={t("oauthCallback")} />}>
        {params.then(({ provider }) => {
          if (!isProvider(provider)) notFound();
          return <OAuthCallback provider={provider} />;
        })}
      </Suspense>
    </Page>
  );
}
