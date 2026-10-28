import { getTranslations } from "next-intl/server";

import {
  hasStarted,
  participantsEditable,
} from "@/features/tournaments/lib/status";
import { loadTournament } from "@/features/tournaments/server";
import { serverApi } from "@/shared/api/server";
import { Avatar } from "@/shared/ui/avatar";
import { DataTable } from "@/shared/ui/data-table";
import { LocalDateTime } from "@/shared/ui/local-date-time";
import { Section } from "@/shared/ui/section";
import { PARTICIPANTS_ID } from "../lib/focus";
import { getParticipants } from "../server";
import { AddParticipantButton } from "./add-participant-button";
import { ParticipantActions } from "./participant-actions";
import { ParticipantState } from "./participant-state";
import styles from "./participant-table.module.scss";

type Props = { tenant: string; slug: string };

/**
 * A aba Inscritos: a tabela com seed (depois de iniciar, na ordem da chave),
 * apelido (link para a pessoa na comunidade), estado e data de inscrição,
 * com busca e paginação. Para a organização, "Inscrever alguém" na barra e,
 * por linha, marcar ou desmarcar o check-in, tirar do torneio (até começar)
 * e marcar a desistência (em andamento). Fica em `<Suspense>`.
 */
export async function ParticipantTable({ tenant, slug }: Props) {
  const loaded = await loadTournament(tenant, slug);
  if (loaded.kind !== "ok") return null;

  const { me, tournament } = loaded;
  const { status, max_participants: max } = tournament;
  const organizer = me.role !== "player";
  const started = hasStarted(status);
  const canManage = organizer && participantsEditable(status);
  const api = await serverApi();
  const [t, participants, members] = await Promise.all([
    getTranslations("Participants"),
    getParticipants(tenant, slug).then((list) => list ?? []),
    canManage
      ? api
          .GET("/tenants/{slug}/members", {
            params: { path: { slug: tenant } },
          })
          .then(({ data }) => data ?? [])
          .catch(() => [])
      : [],
  ]);
  // Depois de iniciar, na ordem da chave (quem ficou de fora, sem seed, no fim).
  const listed = started
    ? participants.toSorted(
        (a, b) => (a.seed ?? Infinity) - (b.seed ?? Infinity),
      )
    : participants;
  const registered = new Set(participants.map((p) => p.user_id));
  const candidates = members
    .filter((m) => !registered.has(m.user_id))
    .map((m) => ({ userId: m.user_id, nickname: m.nickname }));
  const canAdd =
    canManage && candidates.length > 0 && participants.length < max;
  // Antes de abrir, todos estão sem check-in; depois de reabrir a inscrição,
  // os que já fizeram continuam marcados.
  const showCheckIn =
    tournament.check_in_minutes != null &&
    (!["draft", "registration"].includes(status) ||
      participants.some((p) => p.checked_in));

  const rows = listed.map((p) => {
    const checkIn =
      organizer && status === "check_in"
        ? p.checked_in
          ? ("unmark" as const)
          : ("mark" as const)
        : undefined;
    const withdraw =
      organizer && status === "in_progress" && p.seed != null && !p.withdrawn;
    const hasActions = checkIn !== undefined || canManage || withdraw;
    return {
      id: String(p.user_id),
      search: p.nickname,
      // Quem saiu da comunidade não tem mais página nela.
      href: p.left ? undefined : `/${tenant}/members/${p.user_id}`,
      cells: {
        seed: p.seed != null ? t("seed", { seed: p.seed }) : "",
        name: (
          <span className={styles.name}>
            <Avatar src={p.avatar_url} name={p.nickname} />
            {p.nickname}
          </span>
        ),
        state: (
          <ParticipantState
            participant={p}
            showCheckIn={showCheckIn}
            started={started}
          />
        ),
        registeredAt: <LocalDateTime value={p.registered_at} />,
      },
      actions: hasActions ? (
        <ParticipantActions
          tenant={tenant}
          tournament={slug}
          userId={p.user_id}
          nickname={p.nickname}
          checkIn={checkIn}
          remove={canManage}
          withdraw={withdraw}
        />
      ) : undefined,
    };
  });

  return (
    // tabIndex -1: recebe o foco quando a linha da ação some (tirar alguém).
    <div id={PARTICIPANTS_ID} tabIndex={-1}>
      <Section title={t("title", { count: participants.length, max })}>
        <DataTable
          label={t("title", { count: participants.length, max })}
          primaryColumn="name"
          columns={[
            ...(started
              ? [{ key: "seed", header: t("columns.seed"), secondary: true }]
              : []),
            { key: "name", header: t("columns.member") },
            { key: "state", header: t("columns.state") },
            {
              key: "registeredAt",
              header: t("columns.registeredAt"),
              secondary: true,
            },
          ]}
          rows={rows}
          toolbar={
            canAdd ? (
              <AddParticipantButton
                tenant={tenant}
                tournament={slug}
                candidates={candidates}
              />
            ) : undefined
          }
          empty={{ text: canAdd ? t("emptyOrganizer") : t("empty") }}
        />
      </Section>
    </div>
  );
}
