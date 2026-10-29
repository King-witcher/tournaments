import { getTranslations } from "next-intl/server";

import { TOURNAMENT_INVITES_ID } from "@/features/tournaments/lib/focus";
import type { components } from "@/shared/api/schema";
import { serverApi } from "@/shared/api/server";
import { DataTable } from "@/shared/ui/data-table";
import { LocalDateTime } from "@/shared/ui/local-date-time";
import { Notice } from "@/shared/ui/notice";
import { Section } from "@/shared/ui/section";
import { inviteHint } from "../lib/invite-hint";
import { CreateTournamentInviteButton } from "./create-tournament-invite-button";
import { TournamentInviteActions } from "./tournament-invite-actions";

type Props = {
  tenant: string;
  tournament: components["schemas"]["Tournament"];
};

/**
 * Na aba Organização, os links de inscrição do torneio (no rascunho e com a
 * inscrição aberta): gerar (o link aparece uma vez, para copiar) e, por
 * linha, quem criou, quando, quantos entraram por ele e revogar. Fica em
 * `<Suspense>`.
 */
export async function TournamentInviteSection({ tenant, tournament }: Props) {
  const api = await serverApi();
  const [t, invites] = await Promise.all([
    getTranslations("Tournament.invites"),
    api
      .GET("/tenants/{slug}/tournaments/{tournament}/invites", {
        params: { path: { slug: tenant, tournament: tournament.slug } },
      })
      .then(({ data }) => data ?? [])
      .catch(() => []),
  ]);
  // Vagas preenchidas: o link não inscreve ninguém até abrir vaga.
  const full = tournament.participants >= tournament.max_participants;

  return (
    // tabIndex -1: recebe o foco quando a linha do link revogado some.
    <div id={TOURNAMENT_INVITES_ID} tabIndex={-1}>
      <Section
        title={t("title")}
        description={t("description")}
        action={
          <CreateTournamentInviteButton
            tenant={tenant}
            tournament={tournament.slug}
          />
        }
      >
        {full && <Notice kind="warning">{t("full")}</Notice>}
        <DataTable
          label={t("title")}
          size="sm"
          columns={[
            // No celular: o link, as inscrições e revogar, sem rolar.
            { key: "link", header: t("columns.link") },
            {
              key: "createdBy",
              header: t("columns.createdBy"),
              secondary: true,
            },
            {
              key: "createdAt",
              header: t("columns.createdAt"),
              secondary: true,
            },
            { key: "uses", header: t("columns.uses") },
          ]}
          rows={invites.map((invite) => ({
            id: String(invite.id),
            search: `${inviteHint(invite.hint) ?? ""} ${invite.created_by}`,
            cells: {
              link: inviteHint(invite.hint) ?? "—",
              createdBy: invite.created_by,
              createdAt: <LocalDateTime value={invite.created_at} />,
              uses: String(invite.uses),
            },
            actions: (
              <TournamentInviteActions
                tenant={tenant}
                inviteId={invite.id}
                hint={inviteHint(invite.hint)}
              />
            ),
          }))}
          empty={{ text: t("empty") }}
        />
      </Section>
    </div>
  );
}
