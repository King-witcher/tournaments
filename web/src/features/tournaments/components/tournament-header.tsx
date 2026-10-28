import { useTranslations } from "next-intl";

import { Link } from "@/i18n/navigation";
import type { components } from "@/shared/api/schema";
import { LocalDateTime } from "@/shared/ui/local-date-time";

type Props = {
  tenant: string;
  tenantName: string;
  /** Nome já no idioma de quem vê. */
  name: string;
  tournament: components["schemas"]["Tournament"];
};

/** Comunidade, nome, status, visibilidade e início (no fuso de quem vê). */
export function TournamentHeader({
  tenant,
  tenantName,
  name,
  tournament,
}: Props) {
  const t = useTranslations("Tournament");
  const tStatus = useTranslations("TournamentStatus");
  const tVisibility = useTranslations("Visibility");
  const startsAt = tournament.starts_at;
  const started =
    tournament.status === "in_progress" || tournament.status === "finished";

  return (
    <header className="flex flex-col gap-1">
      <Link
        href={`/${tenant}`}
        className="text-sm text-muted underline underline-offset-4 hover:decoration-2"
      >
        {tenantName}
      </Link>
      <h1 className="text-3xl font-semibold">{name}</h1>
      <p className="text-sm text-muted">
        {tStatus(tournament.status)} · {tVisibility(tournament.visibility)}
      </p>
      <p className="text-sm">
        {startsAt
          ? t.rich(started ? "startedAt" : "startsAt", {
              date: () => <LocalDateTime value={startsAt} />,
            })
          : t("noStartDate")}
      </p>
    </header>
  );
}
