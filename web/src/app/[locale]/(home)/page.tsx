import type { Metadata, ResolvingMetadata } from "next";
import { getTranslations } from "next-intl/server";
import { Suspense } from "react";

import { AuthError } from "@/features/auth/components/auth-error";
import { LandingClosing } from "@/features/landing/components/landing-closing";
import { LandingFeatures } from "@/features/landing/components/landing-features";
import { LandingHero } from "@/features/landing/components/landing-hero";
import { LandingSteps } from "@/features/landing/components/landing-steps";
import { LandingTour } from "@/features/landing/components/landing-tour";
import { publicPageMetadata } from "@/shared/lib/link-preview";
import { Page } from "@/shared/ui/page";

export async function generateMetadata(
  _: PageProps<"/[locale]">,
  parent: ResolvingMetadata,
): Promise<Metadata> {
  const t = await getTranslations("Landing");
  return publicPageMetadata(
    "",
    { title: t("hero.title"), description: t("hero.lead") },
    parent,
  );
}

/**
 * O início de quem não entrou: a apresentação do site, estática (sai pronta,
 * sem esperar a API). Com cookie de sessão, o proxy serve `/me` neste
 * endereço.
 */
export default function Landing({ searchParams }: PageProps<"/[locale]">) {
  return (
    <>
      {/* O login pelo provedor que deu errado volta para cá. */}
      <Suspense fallback={null}>
        <Page width="wide">
          <AuthError searchParams={searchParams} />
        </Page>
      </Suspense>
      <LandingHero />
      <LandingSteps />
      <LandingTour />
      <LandingFeatures />
      <LandingClosing />
    </>
  );
}
