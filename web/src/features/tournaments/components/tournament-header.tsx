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
  /** Referência para saber se a data de início já passou. */
  now: Date;
};

/** Comunidade, nome, status, visibilidade e início (no fuso de quem vê). */
export function TournamentHeader({
  tenant,
  tenantName,
  name,
  tournament,
  now,
}: Props) {
  const t = useTranslations("Tournament");
  const tStatus = useTranslations("TournamentStatus");
  const tVisibility = useTranslations("Visibility");
  const startsAt = tournament.starts_at;
  // Data que já passou (começou, ou a organização ainda não iniciou): "início
  // marcado para", não "começa", que daria a entender que ainda vai começar.
  const past =
    tournament.status === "in_progress" ||
    tournament.status === "finished" ||
    (startsAt != null && new Date(startsAt) <= now);

  return (
    <header className="flex flex-col gap-1">
      <Link
        href={`/${tenant}`}
        className="inline-flex min-h-11 items-center self-start text-sm text-muted underline underline-offset-4 hover:decoration-2 sm:min-h-0"
      >
        {tenantName}
      </Link>
      <h1 className="text-3xl font-semibold">{name}</h1>
      <p className="text-sm text-muted">
        {tStatus(tournament.status)} · {tVisibility(tournament.visibility)}
      </p>
      <p className="text-sm">
        {startsAt
          ? t.rich(past ? "startedAt" : "startsAt", {
              date: () => <LocalDateTime value={startsAt} />,
            })
          : t("noStartDate")}
      </p>
    </header>
  );
}
