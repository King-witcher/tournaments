import { getTranslations } from "next-intl/server";

import { LinkTile } from "@/shared/ui/link-tile";
import { TileGrid } from "@/shared/ui/tile-grid";

/** A entrada da sala de veto na coluna lateral do início de quem entrou. */
export async function VetoRoomEntry() {
  const t = await getTranslations("VetoRoom");
  return (
    <TileGrid>
      <LinkTile href="/veto" title={t("title")} meta={t("description")} />
    </TileGrid>
  );
}
