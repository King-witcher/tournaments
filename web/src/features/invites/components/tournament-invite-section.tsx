import { getTranslations } from "next-intl/server";

import { serverApi } from "@/shared/api/server";
import { CreateTournamentInviteButton } from "./create-tournament-invite-button";
import { TournamentInviteRow } from "./tournament-invite-row";

type Props = { tenant: string; tournament: string };

/**
 * Links de inscrição do torneio, para organizer e owner, no rascunho ou com a
 * inscrição aberta. Fica em `<Suspense>`.
 */
export async function TournamentInviteSection({ tenant, tournament }: Props) {
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
      <p className="text-sm opacity-70">{t("description")}</p>
      <CreateTournamentInviteButton tenant={tenant} tournament={tournament} />
      {invites.length === 0 ? (
        <p className="text-sm opacity-70">{t("empty")}</p>
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
