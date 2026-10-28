import { ButtonSkeleton, SkeletonText, Stack, Tile } from "@carbon/react";

import { PageHeaderSkeleton } from "@/shared/ui/skeleton";

/**
 * A página do convite carregando, na forma dela: o título e o `Tile` com o
 * que o convite é e o botão de aceitar.
 */
export function InviteSkeleton({ label }: { label: string }) {
  return (
    <>
      <PageHeaderSkeleton breadcrumb={false} label={label} />
      <div aria-hidden>
        <Tile>
          <Stack gap={6}>
            <SkeletonText paragraph lineCount={3} />
            <ButtonSkeleton />
          </Stack>
        </Tile>
      </div>
    </>
  );
}
