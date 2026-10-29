import { getLocale, getTranslations } from "next-intl/server";

import { getTenant } from "@/features/tenants/server";
import type { components } from "@/shared/api/schema";
import { serverApi } from "@/shared/api/server";
import { ButtonLink } from "@/shared/ui/button-link";
import { DataTable } from "@/shared/ui/data-table";
import { LocalDateTime } from "@/shared/ui/local-date-time";
import { StatusTag } from "@/shared/ui/status-tag";
import { checkInPending } from "../lib/check-in";
import { localizedName } from "../lib/localize";
import styles from "./tournament-list.module.scss";

type Schemas = components["schemas"];

/**
 * Do que está acontecendo para o que já acabou; dentro de cada status, a
 * ordem da API (mais novo primeiro).
 */
const ORDER: Schemas["TournamentStatus"][] = [
  "in_progress",
  "check_in",
  "registration",
  "draft",
  "finished",
];

/**
 * A aba Torneios da comunidade: a tabela (nome, formato, estado, início,
 * inscritos), do que está acontecendo para o que acabou. O jogador só recebe
 * os publicados (a API filtra); a organização vê os rascunhos e, sem
 * nenhum, o "Criar torneio". Fica em `<Suspense>`.
 */
export async function TournamentList({ slug }: { slug: string }) {
  const api = await serverApi();
  const [loaded, t, tStatus, tVisibility, locale, tournaments] =
    await Promise.all([
      getTenant(slug),
      getTranslations("Tournaments"),
      getTranslations("TournamentStatus"),
      getTranslations("Visibility"),
      getLocale(),
      api
        .GET("/tenants/{slug}/tournaments", { params: { path: { slug } } })
        .then(({ data }) =>
          (data ?? []).toSorted(
            (a, b) => ORDER.indexOf(a.status) - ORDER.indexOf(b.status),
          ),
        ),
    ]);
  const organizer = loaded.kind === "ok" && loaded.tenant.role !== "player";

  return (
    <DataTable
      label={t("label")}
      columns={[
        { key: "name", header: t("columns.name") },
        { key: "format", header: t("columns.format"), secondary: true },
        { key: "status", header: t("columns.status") },
        { key: "starts", header: t("columns.starts"), secondary: true },
        {
          key: "participants",
          header: t("columns.participants"),
          secondary: true,
        },
      ]}
      rows={tournaments.map((tournament) => {
        const name = localizedName(tournament, locale);
        return {
          id: tournament.slug,
          href: `/${slug}/${tournament.slug}`,
          search: `${name} ${tournament.name}`,
          cells: {
            name,
            // O mesmo texto do `useModeLabel` ("Duelo", "TDM 2v2").
            format:
              tournament.mode === "tdm"
                ? t("modeTeams", { size: tournament.team_size })
                : t(`mode.${tournament.mode}`),
            status: (
              <span className={styles.tags}>
                <StatusTag status={tournament.status}>
                  {tStatus(tournament.status)}
                </StatusTag>
                {checkInPending(tournament) ? (
                  <StatusTag status="check_in">{t("checkInPending")}</StatusTag>
                ) : (
                  tournament.registered && (
                    <StatusTag status="registration">
                      {t("registered")}
                    </StatusTag>
                  )
                )}
                {tournament.visibility === "private" && (
                  <StatusTag status="neutral">
                    {tVisibility("private")}
                  </StatusTag>
                )}
              </span>
            ),
            starts: tournament.starts_at ? (
              <LocalDateTime value={tournament.starts_at} />
            ) : (
              t("noDate")
            ),
            participants: t("participants", {
              count: tournament.participants,
              max: tournament.max_participants,
            }),
          },
        };
      })}
      empty={
        organizer
          ? {
              text: t("empty"),
              // O "Novo torneio" primário já está no cabeçalho.
              action: (
                <ButtonLink href={`/${slug}/new`} kind="tertiary">
                  {t("create")}
                </ButtonLink>
              ),
            }
          : { text: t("emptyPlayer") }
      }
    />
  );
}
