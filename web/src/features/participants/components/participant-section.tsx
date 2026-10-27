import { getTranslations } from "next-intl/server";

import { serverApi } from "@/shared/api/server";
import { AddParticipantForm } from "./add-participant-form";
import { ParticipantRow } from "./participant-row";

type Props = {
  tenant: string;
  tournament: string;
  /** Vagas do torneio. */
  max: number;
  /** Organizer ou owner, com o torneio ainda sem começar: inscreve e tira membros. */
  canManage: boolean;
};

/** Inscritos do torneio e, para quem gerencia, inscrever membros. Fica em `<Suspense>`. */
export async function ParticipantSection({
  tenant,
  tournament,
  max,
  canManage,
}: Props) {
  const api = await serverApi();
  const [t, participants, members] = await Promise.all([
    getTranslations("Participants"),
    api
      .GET("/tenants/{slug}/tournaments/{tournament}/participants", {
        params: { path: { slug: tenant, tournament } },
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

  return (
    <section className="flex flex-col gap-3">
      <h2 className="text-xl font-semibold">
        {t("title", { count: participants.length, max })}
      </h2>
      {canManage && candidates.length > 0 && participants.length < max && (
        <AddParticipantForm
          tenant={tenant}
          tournament={tournament}
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
              tournament={tournament}
              participant={participant}
              canRemove={canManage}
            />
          ))}
        </ul>
      )}
    </section>
  );
}
