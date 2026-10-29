import { useLocale, useTranslations } from "next-intl";

import { localizedName } from "@/features/tournaments/lib/localize";
import type { components } from "@/shared/api/schema";
import { LinkTile } from "@/shared/ui/link-tile";
import { LocalDateTime } from "@/shared/ui/local-date-time";
import { StatusTag } from "@/shared/ui/status-tag";
import { PendingMeta } from "./pending-meta";

/**
 * Um torneio cujo início marcado chegou sem ser iniciado, como tile que
 * leva à aba Organização (onde se inicia, com o seeding, ou se muda a data):
 * o torneio, a comunidade, o início marcado e quantos entrariam na chave.
 */
export function StartDueTile({
  item,
}: {
  item: components["schemas"]["StartDue"];
}) {
  const t = useTranslations("Organizing.startDue");
  const locale = useLocale();
  const { tournament } = item;

  return (
    <LinkTile
      href={`/${tournament.tenant_slug}/${tournament.slug}/manage`}
      title={localizedName(tournament, locale)}
      meta={
        <PendingMeta
          lines={[
            tournament.tenant_name,
            t.rich("startedAt", {
              date: () => <LocalDateTime value={item.starts_at} />,
            }),
            item.has_check_in
              ? t("checkedIn", {
                  checkedIn: item.checked_in,
                  registered: item.registered,
                })
              : t("registered", { registered: item.registered }),
          ]}
        />
      }
      tags={<StatusTag status="waiting">{t("tag")}</StatusTag>}
    />
  );
}
