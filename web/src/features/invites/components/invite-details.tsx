import { useTranslations } from "next-intl";
import type { ReactNode } from "react";

import type { components } from "@/shared/api/schema";
import { LocalDateTime } from "@/shared/ui/local-date-time";
import { StructuredList } from "@/shared/ui/structured-list";

type Props = {
  invite: components["schemas"]["InvitePreview"];
};

/**
 * O que o convite é, em rótulo e valor: a comunidade e, no link de
 * inscrição, as vagas e o início; a validade, quando ele vence.
 */
export function InviteDetails({ invite }: Props) {
  const t = useTranslations("Invite");
  const { tournament } = invite;
  const rows: { key: string; cells: ReactNode[] }[] = [
    { key: "community", cells: [t("community"), invite.tenant_name] },
  ];
  if (tournament) {
    rows.push({
      key: "spots",
      cells: [
        t("spots"),
        t("tournamentSpots", {
          count: tournament.participants,
          max: tournament.max_participants,
        }),
      ],
    });
    if (tournament.starts_at) {
      rows.push({
        key: "starts",
        cells: [
          t("startsAt"),
          <LocalDateTime key="value" value={tournament.starts_at} />,
        ],
      });
    }
  }
  if (invite.expires_at) {
    rows.push({
      key: "expires",
      cells: [
        t("expiresAt"),
        <LocalDateTime key="value" value={invite.expires_at} />,
      ],
    });
  }

  return <StructuredList label={t("details")} rows={rows} condensed />;
}
