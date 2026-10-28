import { useTranslations } from "next-intl";

import { Link } from "@/i18n/navigation";
import type { components } from "@/shared/api/schema";
import { buttonVariants } from "@/shared/ui/button";
import { DeleteTournamentButton } from "./delete-tournament-button";
import { PublishTournamentButton } from "./publish-tournament-button";

type Props = {
  tenant: string;
  tournament: components["schemas"]["Tournament"];
};

/** Para organizer e owner: editar e, no rascunho, publicar ou excluir. */
export function TournamentActions({ tenant, tournament }: Props) {
  const t = useTranslations("Tournament");
  const draft = tournament.status === "draft";
  const undated = !tournament.starts_at;

  return (
    <div className="flex flex-col gap-2">
      {draft && <p className="text-sm opacity-70">{t("draftHint")}</p>}
      {draft && undated && (
        <p className="text-sm opacity-70">{t("publishNeedsDate")}</p>
      )}
      <div className="flex flex-wrap items-start gap-3">
        <Link
          href={`/${tenant}/${tournament.slug}/edit`}
          className={buttonVariants({ variant: "outline", size: "sm" })}
        >
          {t("edit")}
        </Link>
        {draft && (
          <>
            <PublishTournamentButton
              tenant={tenant}
              slug={tournament.slug}
              disabled={undated}
            />
            <DeleteTournamentButton tenant={tenant} slug={tournament.slug} />
          </>
        )}
      </div>
    </div>
  );
}
