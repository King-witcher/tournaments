import { getTranslations } from "next-intl/server";
import { Fragment, type ReactNode } from "react";

import { getMyTenants } from "@/features/tenants/server";
import { MyTaskNotice } from "@/features/tournaments/components/my-task-notice";
import {
  byUrgency,
  playerTask,
  waitsForViewer,
} from "@/features/tournaments/lib/my-tournaments";
import { getMyTournaments } from "@/features/tournaments/server";
import { serverApi } from "@/shared/api/server";
import { TaskNoticeList } from "@/shared/ui/task-notice-list";
import { type Severity, severity, warningsFirst } from "../lib/severity";
import { ContestedGameNotice } from "./contested-game-notice";
import { PendingTiesNotice } from "./pending-ties-notice";
import { StageDueNotice } from "./stage-due-notice";
import { StartDueNotice } from "./start-due-notice";

/**
 * O topo da home: o que só anda se quem vê agir, em todas as comunidades. Um
 * aviso por pendência, as exceções primeiro (`warningsFirst`) e, dentro de
 * cada grupo, o que ele tem de fazer como jogador, que corre contra o relógio,
 * e depois o que espera a organização (os mapas contestados, em que há uma
 * partida parada, os torneios na hora de iniciar, os com uma fase pronta para
 * iniciar e os que esperam a decisão de um empate na classificação). Nada
 * esperando, nada aparece: o aviso com som e o push já chamam quando chega
 * algo, e a home recarrega sozinha pelo canal da pessoa. Fica em `<Suspense>`.
 *
 * `focusAfter`: o alvo do foco quando o check-in feito aqui some da lista.
 * `tenant`: só o desta comunidade (o início dela). Os avisos não dizem o
 * torneio nem a comunidade: quem vê descobre ao abrir.
 */
export async function WaitingForYou({
  focusAfter,
  tenant,
}: {
  focusAfter: string;
  tenant?: string;
}) {
  const [tenants, mine] = await Promise.all([
    getMyTenants(),
    getMyTournaments(),
  ]);
  if (!tenants) return null;
  // Só quem organiza pergunta à API: na home de quem só joga, a rota
  // custaria mais uma leitura da sessão (Redis) a cada recarga.
  const api = await serverApi();
  const here = (slug: string) => tenant === undefined || slug === tenant;
  const organizing = tenants.some(
    (item) => here(item.slug) && item.role !== "player",
  )
    ? await api
        .GET("/me/organizing")
        .then(({ data }) => data)
        .catch(() => undefined)
    : undefined;

  const tasks = (mine?.registered ?? [])
    .filter((entry) => here(entry.tenant_slug) && waitsForViewer(entry))
    .toSorted(byUrgency);
  const ofHere = <T extends { tournament: { tenant_slug: string } }>(
    items: T[] | undefined,
  ) => (items ?? []).filter((item) => here(item.tournament.tenant_slug));

  const notice = (
    kind: Severity,
    key: string,
    render: (kind: Severity) => ReactNode,
  ) => ({ kind, key, node: render(kind) });

  const notices = warningsFirst([
    ...tasks.map((entry) =>
      notice(
        severity(playerTask(entry) ?? "check_in"),
        `${entry.tenant_slug}/${entry.tournament.slug}`,
        (kind) => (
          <MyTaskNotice kind={kind} entry={entry} focusAfter={focusAfter} />
        ),
      ),
    ),
    ...ofHere(organizing?.contested).map((item) =>
      notice(
        severity("resolve"),
        `${item.tournament.tenant_slug}/${item.tournament.slug}/${item.key}/${item.game}`,
        (kind) => <ContestedGameNotice kind={kind} item={item} />,
      ),
    ),
    ...ofHere(organizing?.start_due).map((item) =>
      notice(
        severity("start_due"),
        `${item.tournament.tenant_slug}/${item.tournament.slug}`,
        (kind) => <StartDueNotice kind={kind} item={item} />,
      ),
    ),
    ...ofHere(organizing?.stage_due).map((item) =>
      notice(
        severity("stage_due"),
        `${item.tournament.tenant_slug}/${item.tournament.slug}/${item.stage}`,
        (kind) => <StageDueNotice kind={kind} item={item} />,
      ),
    ),
    ...ofHere(organizing?.pending_ties).map((item) =>
      notice(
        severity("pending_ties"),
        `${item.tournament.tenant_slug}/${item.tournament.slug}/ties`,
        (kind) => <PendingTiesNotice kind={kind} item={item} />,
      ),
    ),
  ]);
  if (notices.length === 0) return null;
  const t = await getTranslations("Home.waiting");

  return (
    <TaskNoticeList label={t("title")}>
      {notices.map(({ key, node }) => (
        <Fragment key={key}>{node}</Fragment>
      ))}
    </TaskNoticeList>
  );
}
