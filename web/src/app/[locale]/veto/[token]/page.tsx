import type { Metadata } from "next";
import { Suspense } from "react";

import { VetoRoomView } from "@/features/veto-rooms/components/veto-room-view";
import { vetoRoomMetadata } from "@/features/veto-rooms/server";
import { Page } from "@/shared/ui/page";
import { PageSkeleton } from "@/shared/ui/skeleton";

export async function generateMetadata({
  params,
}: PageProps<"/[locale]/veto/[token]">): Promise<Metadata> {
  return vetoRoomMetadata((await params).token);
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
