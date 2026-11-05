import { ButtonSkeleton } from "@/shared/ui/carbon/button";
import { SkeletonText } from "@/shared/ui/carbon/skeleton-text";
import { Stack } from "@/shared/ui/carbon/stack";
import { Tile } from "@/shared/ui/carbon/tile";
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
