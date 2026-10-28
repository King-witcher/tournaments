import { getTranslations } from "next-intl/server";

import { serverApi } from "@/shared/api/server";
import { CreateTournamentInviteButton } from "./create-tournament-invite-button";
import { TournamentInviteRow } from "./tournament-invite-row";

type Props = {
  tenant: string;
  tournament: string;
  /** Vagas preenchidas: o link não inscreve ninguém até abrir vaga. */
  full: boolean;
};

/**
 * Links de inscrição do torneio, para organizer e owner, no rascunho ou com a
 * inscrição aberta. Fica em `<Suspense>`.
 */
export async function TournamentInviteSection({
  tenant,
  tournament,
  full,
}: Props) {
  const api = await serverApi();
  const [t, invites] = await Promise.all([
    getTranslations("Tournament.invites"),
    api
      .GET("/tenants/{slug}/tournaments/{tournament}/invites", {
        params: { path: { slug: tenant, tournament } },
      })
      .then(({ data }) => data ?? []),
  ]);
  const now = new Date();

  return (
    <section className="flex flex-col gap-3">
      <h2 className="text-xl font-semibold">{t("title")}</h2>
      <p className="text-sm text-muted">{t("description")}</p>
      {full && <p className="text-sm">{t("full")}</p>}
      <CreateTournamentInviteButton tenant={tenant} tournament={tournament} />
      {invites.length === 0 ? (
        <p className="text-sm text-muted">{t("empty")}</p>
      ) : (
        <ul className="divide-y divide-black/10 dark:divide-white/10">
          {invites.map((invite) => (
            <TournamentInviteRow
              key={invite.id}
              tenant={tenant}
              invite={invite}
              now={now}
            />
          ))}
        </ul>
      )}
    </section>
  );
}
