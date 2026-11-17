import { getTranslations } from "next-intl/server";

import { getCurrentUser } from "@/features/auth/server";
import { apiErrorMessage } from "@/shared/api/errors";
import type { components } from "@/shared/api/schema";
import { Avatar } from "@/shared/ui/avatar";
import { Column, Grid } from "@/shared/ui/carbon/grid";
import { Stack } from "@/shared/ui/carbon/stack";
import { DataTable } from "@/shared/ui/data-table";
import { LocalDateTime } from "@/shared/ui/local-date-time";
import { Notice } from "@/shared/ui/notice";
import { Panel } from "@/shared/ui/panel";
import { SectionSkeleton, TableSkeleton } from "@/shared/ui/skeleton";
import { shownNickname } from "../lib/nickname";
import type { Modality } from "../lib/opponents-href";
import { getMember, getMemberOpponents } from "../server";
import styles from "./member-opponents-tab.module.scss";
import { ModalitySwitcher } from "./modality-switcher";
import nameStyles from "./opponent-name.module.scss";
import { PendingContent } from "./pending-content";
import { SearchTransition } from "./search-transition";
import { VersusPanel } from "./versus-panel";

type Props = {
  tenant: string;
  /** O `user_id` do endereço, ainda como texto. */
  user: string;
  modality?: Modality;
};

type Record = components["schemas"]["SeriesRecord"];

/**
 * A aba Confrontos: a modalidade em cima e, no perfil de outra pessoa,
 * "Você contra {apelido}" em destaque; embaixo, uma linha por pessoa que
 * esteve do outro lado, com séries e partidas V–D e o último confronto (a
 * linha abre o perfil dela; quem saiu não tem). Sem o membro, nada (a casca
 * já mostra o erro). Fica em `<Suspense>`.
 */
export async function MemberOpponentsTab({ tenant, user, modality }: Props) {
  const member = await getMember(tenant, user);
  if (!member.ok) return null;
  const { user_id: userId, nickname } = member.data;
  const [loaded, me, t, tMember, tLoading] = await Promise.all([
    getMemberOpponents(tenant, userId, modality),
    getCurrentUser(),
    getTranslations("Member.opponents"),
    getTranslations("Member"),
    getTranslations("Member.loadingTab"),
  ]);
  if (!loaded.ok) {
    return <Notice kind="error">{await apiErrorMessage(loaded.code)}</Notice>;
  }
  const record = (r: Record) =>
    r.draws > 0
      ? tMember("career.recordDraws", r)
      : tMember("career.record", { wins: r.wins, losses: r.losses });
  const { opponents } = loaded.data;
  const isYou = me?.id === userId;
  const mine = opponents.find((o) => o.person.user_id === me?.id);

  // O destaque e a tabela mudam com a modalidade: esqueleto na troca.
  const versus = !isYou && me != null;
  return (
    <SearchTransition>
      <Stack gap={6}>
        <div className={styles.modality}>
          <ModalitySwitcher
            base={`/${tenant}/members/${userId}`}
            value={modality ?? "all"}
          />
        </div>
        <PendingContent
          label={tLoading("opponents")}
          fallback={
            <>
              {versus && (
                <Panel>
                  <SectionSkeleton rows={2} />
                </Panel>
              )}
              <TableSkeleton columns={4} rows={8} />
            </>
          }
        >
          {versus && (
            <Grid>
              <Column sm={4} md={8} lg={8}>
                <VersusPanel nickname={nickname} record={mine} />
              </Column>
            </Grid>
          )}
          <DataTable
            label={t("label")}
            columns={[
              { key: "opponent", header: t("opponent") },
              { key: "series", header: t("series") },
              { key: "games", header: t("games"), secondary: true },
              { key: "last", header: t("last"), secondary: true },
            ]}
            rows={opponents.map(({ person, series, games, last_at }) => {
              const name = shownNickname(
                person.nickname,
                tMember("deletedAccount"),
              );
              return {
                id: String(person.user_id),
                href: person.active
                  ? `/${tenant}/members/${person.user_id}`
                  : undefined,
                search: name,
                cells: {
                  opponent: (
                    <span className={nameStyles.opponent}>
                      <Avatar src={person.avatar_url} name={name} />
                      {person.active || person.nickname === ""
                        ? name
                        : tMember("left", { name })}
                    </span>
                  ),
                  series: record(series),
                  games: tMember("career.record", games),
                  last: <LocalDateTime value={last_at} />,
                },
              };
            })}
            empty={{ text: t("empty") }}
          />
        </PendingContent>
      </Stack>
    </SearchTransition>
  );
}
