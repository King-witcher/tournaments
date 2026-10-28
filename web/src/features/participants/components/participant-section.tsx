import { getTranslations } from "next-intl/server";

import { participantsEditable } from "@/features/tournaments/lib/status";
import type { components } from "@/shared/api/schema";
import { serverApi } from "@/shared/api/server";
import { AddParticipantForm } from "./add-participant-form";
import { ParticipantRow } from "./participant-row";

type Props = {
  tenant: string;
  tournament: components["schemas"]["Tournament"];
  /** Organizer ou owner. */
  organizer: boolean;
};

/**
 * Inscritos do torneio, com check-in e (depois de iniciar) seed; para quem
 * gerencia, inscrever, tirar e marcar o check-in. Fica em `<Suspense>`.
 */
export async function ParticipantSection({
  tenant,
  tournament,
  organizer,
}: Props) {
  const { slug, status, max_participants: max } = tournament;
  const beforeStart = participantsEditable(status);
  const canManage = organizer && beforeStart;
  const api = await serverApi();
  const [t, participants, members] = await Promise.all([
    getTranslations("Participants"),
    api
      .GET("/tenants/{slug}/tournaments/{tournament}/participants", {
        params: { path: { slug: tenant, tournament: slug } },
      })
      .then(({ data }) => data ?? []),
    canManage
      ? api
          .GET("/tenants/{slug}/members", {
            params: { path: { slug: tenant } },
          })
          .then(({ data }) => data ?? [])
      : [],
  ]);
  const registered = new Set(participants.map((p) => p.user_id));
  const candidates = members
    .filter((m) => !registered.has(m.user_id))
    .map((m) => ({ userId: m.user_id, nickname: m.nickname }));
  // Antes de abrir, todos estão sem check-in; depois de reabrir a inscrição,
  // os que já fizeram continuam marcados.
  const showCheckIn =
    tournament.check_in_minutes != null &&
    (!["draft", "registration"].includes(status) ||
      participants.some((p) => p.checked_in));

  return (
    <section className="flex flex-col gap-3">
      <h2 className="text-xl font-semibold">
        {t("title", { count: participants.length, max })}
      </h2>
      {canManage && candidates.length > 0 && participants.length < max && (
        <AddParticipantForm
          tenant={tenant}
          tournament={slug}
          candidates={candidates}
        />
      )}
      {participants.length === 0 ? (
        <p className="text-sm opacity-70">{t("empty")}</p>
      ) : (
        <ul className="divide-y divide-black/10 dark:divide-white/10">
          {participants.map((participant) => (
            <ParticipantRow
              key={participant.user_id}
              tenant={tenant}
              tournament={slug}
              participant={participant}
              showCheckIn={showCheckIn}
              showSeed={!beforeStart}
              canCheckIn={organizer && status === "check_in"}
              canRemove={canManage}
            />
          ))}
        </ul>
      )}
    </section>
  );
}
