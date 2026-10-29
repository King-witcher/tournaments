import { getLocale, getTranslations } from "next-intl/server";

import { LoginPrompt } from "@/features/auth/components/login-prompt";
import { ErrorBack } from "@/features/tournaments/components/error-back";
import { localizedName } from "@/features/tournaments/lib/localize";
import {
  hasStarted,
  participantsEditable,
  seedingOpen,
} from "@/features/tournaments/lib/status";
import { loadTournament } from "@/features/tournaments/server";
import { apiErrorMessage } from "@/shared/api/errors";
import { Link } from "@/shared/ui/link";
import { PageError } from "@/shared/ui/page-error";
import { PageHeader } from "@/shared/ui/page-header";
import { checkInShown } from "../lib/formation";
import { getFormation } from "../server";
import { FormationEditor } from "./formation-editor";
import { FormationSummary } from "./formation-summary";
import { TeamsSection } from "./teams-section";
import styles from "./teams-view.module.scss";

type Props = { tenant: string; slug: string };

/**
 * A página dos times do TDM. Para a organização, antes de iniciar, a
 * formação editável (`FormationEditor`); depois de iniciar ou para quem
 * joga, só leitura, com o "Renomear" do capitão. No duelo não há times: o
 * aviso e o caminho de volta. Lê a sessão, então fica em `<Suspense>`.
 */
export async function TeamsView({ tenant, slug }: Props) {
  const [t, tNav, tTournament, locale, loaded] = await Promise.all([
    getTranslations("Team"),
    getTranslations("Nav"),
    getTranslations("Tournament"),
    getLocale(),
    loadTournament(tenant, slug),
  ]);
  if (loaded.kind === "login") {
    return <LoginPrompt message={tTournament("loginRequired")} />;
  }
  if (loaded.kind === "error") {
    return <ErrorBack tenant={tenant} code={loaded.code} />;
  }

  const { me, tournament } = loaded;
  const path = `/${tenant}/${slug}`;
  const header = (
    <PageHeader
      breadcrumbs={[
        { href: "/", label: tNav("home") },
        { href: `/${tenant}`, label: me.name },
        { href: path, label: localizedName(tournament, locale) },
      ]}
      title={t("pageTitle")}
    >
      {tournament.mode === "tdm" && (
        <p className={styles.hint}>
          {t("what", { size: tournament.team_size })}
        </p>
      )}
    </PageHeader>
  );

  const formation =
    tournament.mode === "tdm" ? await getFormation(tenant, slug) : undefined;
  if (!formation) {
    // No duelo não há times; no TDM, a API falhou.
    const code = tournament.mode === "tdm" ? "internal" : "tournament.not_tdm";
    return (
      <>
        {header}
        <PageError href={path} label={tNav("backToTournament")}>
          {await apiErrorMessage(code)}
        </PageError>
      </>
    );
  }

  const { status } = tournament;
  const organizer = me.role !== "player";
  const checkIn = checkInShown(status, formation);
  const links = organizer && (
    <>
      {seedingOpen(tournament) && (
        <Link href={`${path}/seeding`}>{t("seeding")}</Link>
      )}
      <Link href={`${path}/manage`}>{t("backToManage")}</Link>
    </>
  );

  if (organizer && participantsEditable(status)) {
    return (
      <>
        {header}
        <FormationEditor
          tenant={tenant}
          tournament={slug}
          initial={formation}
          checkIn={checkIn}
          links={links}
        />
      </>
    );
  }
  return (
    <>
      {header}
      <FormationSummary
        formation={formation}
        checkIn={checkIn}
        started={hasStarted(status)}
        links={links}
      />
      <TeamsSection tenant={tenant} slug={slug} />
    </>
  );
}
