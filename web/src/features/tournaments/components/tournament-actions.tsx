import { useTranslations } from "next-intl";

import { LifecyclePanel } from "@/features/lifecycle/components/lifecycle-panel";
import { ORGANIZER_PANEL_ID } from "@/features/lifecycle/lib/organizer-panel";
import { Link } from "@/i18n/navigation";
import type { components } from "@/shared/api/schema";
import { buttonVariants } from "@/shared/ui/button";
import { textsEditable } from "../lib/status";
import { DeleteTournamentButton } from "./delete-tournament-button";
import { PublishTournamentButton } from "./publish-tournament-button";

type Props = {
  tenant: string;
  tournament: components["schemas"]["Tournament"];
  /** Referência para saber se a abertura do check-in já passou. */
  now: Date;
};

/**
 * Para organizer e owner, num quadro só no topo da página: em que pé está o
 * torneio e o próximo passo (publicar, abrir o check-in, iniciar, desfazer),
 * e editar. No rascunho, publicar e excluir.
 */
export function TournamentActions({ tenant, tournament, now }: Props) {
  const t = useTranslations("Tournament");
  const draft = tournament.status === "draft";
  const undated = !tournament.starts_at;
  // Pela data e pela janela, o check-in já abriu: publicado agora, o torneio
  // vai direto para o check-in, sem inscrições.
  const opensAt = tournament.check_in_opens_at;
  const skipsRegistration = opensAt != null && new Date(opensAt) <= now;
  const edit = (
    <Link
      href={`/${tenant}/${tournament.slug}/edit`}
      className={buttonVariants({ variant: "outline", size: "sm" })}
    >
      {t("edit")}
    </Link>
  );

  return (
    // tabIndex -1: recebe o foco depois de publicar, abrir o check-in,
    // iniciar ou desfazer, quando o botão usado some.
    <section
      id={ORGANIZER_PANEL_ID}
      tabIndex={-1}
      className="flex flex-col gap-2 rounded-md border border-black/10 p-3 dark:border-white/15"
    >
      <h2 className="font-semibold">{t("organizerTitle")}</h2>
      {draft ? (
        <>
          <p className="text-sm">{t("draftHint")}</p>
          {undated && (
            <p className="text-sm text-muted">{t("publishNeedsDate")}</p>
          )}
          {skipsRegistration && (
            <p className="text-sm">{t("publishSkipsRegistration")}</p>
          )}
          <div className="flex flex-wrap items-start gap-3">
            <PublishTournamentButton
              tenant={tenant}
              slug={tournament.slug}
              disabled={undated}
              skipsRegistration={skipsRegistration}
            />
            {edit}
            <DeleteTournamentButton tenant={tenant} slug={tournament.slug} />
          </div>
        </>
      ) : (
        <LifecyclePanel
          tenant={tenant}
          tournament={tournament}
          now={now}
          // Encerrado, nada mais muda (só a correção de resultados, na partida).
          extra={textsEditable(tournament.status) ? edit : undefined}
        />
      )}
    </section>
  );
}
