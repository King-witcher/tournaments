import type { Metadata } from "next";
import { getTranslations } from "next-intl/server";
import { Suspense } from "react";

import { VetoRoomView } from "@/features/veto-rooms/components/veto-room-view";
import { Page } from "@/shared/ui/page";
import { PageSkeleton } from "@/shared/ui/skeleton";

/**
 * O token está na URL: nada de indexar, de mandar o endereço a outro site nem
 * de pôr nome ou token na prévia do link. O cabeçalho das mesmas regras está no
 * `next.config.ts`.
 */
export async function generateMetadata(): Promise<Metadata> {
  const t = await getTranslations("VetoRoom");
  return {
    title: t("title"),
    description: t("description"),
    robots: { index: false, follow: false },
    referrer: "no-referrer",
  };
}

export default function VetoRoomPage({
  params,
}: PageProps<"/[locale]/veto/[token]">) {
  return (
    <Page width="wide">
      <Suspense fallback={<PageSkeleton />}>
        {params.then(({ token }) => (
          <VetoRoomView token={token} />
        ))}
      </Suspense>
    </Page>
  );
}
