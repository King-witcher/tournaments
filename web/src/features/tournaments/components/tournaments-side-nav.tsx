import { getLocale, getTranslations } from "next-intl/server";
import type { ReactNode } from "react";

import { getTenant } from "@/features/tenants/server";
import { serverApi } from "@/shared/api/server";
import { AppSideNavTitle } from "@/shared/ui/app-side-nav-title";
import { StatusTag } from "@/shared/ui/status-tag";
import { localizedName } from "../lib/localize";
import { SETTINGS_SECTIONS } from "../lib/settings-sections";
import { hasStarted } from "../lib/status";
import { tournamentTabs } from "../lib/tabs";
import { TournamentSideNavSwitch } from "./tournament-side-nav-switch";

type Props = {
  tenant: string;
  /** O menu da comunidade, que fica quando o endereço não é de um torneio. */
  children: ReactNode;
};

/**
 * O menu lateral dentro de um torneio, no lugar do da comunidade, como no
 * IBM Cloud ao entrar num recurso: a volta à comunidade, "Torneio", o nome e
 * o estado, e as seções. As seções saem da lista (sem as fases, a
 * Classificação aparece desde o início) até a página do torneio dizer as
 * exatas (`TournamentNavRegistration`). Fica em `<Suspense>`.
 */
export async function TournamentsSideNav({ tenant, children }: Props) {
  const api = await serverApi();
  const [loaded, t, tStatus, tSettings, locale, tournaments] =
    await Promise.all([
      getTenant(tenant),
      getTranslations("Tournament"),
      getTranslations("TournamentStatus"),
      getTranslations("TournamentSettings"),
      getLocale(),
      api
        .GET("/tenants/{slug}/tournaments", {
          params: { path: { slug: tenant } },
        })
        .then(({ data }) => data ?? [])
        .catch(() => []),
    ]);
  if (loaded.kind !== "ok") return children;
  const organizer = loaded.tenant.role !== "player";

  const items = tournaments.map((x) => ({
    slug: x.slug,
    header: (
      <AppSideNavTitle
        kind={t("kind")}
        name={localizedName(x, locale)}
        tags={
          <StatusTag status={x.status} size="sm">
            {tStatus(x.status)}
          </StatusTag>
        }
      />
    ),
    tabs: tournamentTabs(`/${tenant}/${x.slug}`, {
      status: x.status,
      standings: hasStarted(x.status),
      organizer,
      teams: x.mode === "tdm",
    }).map((tab) => ({ href: tab.href, label: t(`tabs.${tab.key}`) })),
  }));

  return (
    <TournamentSideNavSwitch
      tenant={tenant}
      label={t("nav")}
      back={{ href: `/${tenant}`, label: loaded.tenant.name }}
      items={items}
      settingsSections={SETTINGS_SECTIONS.map((section) => ({
        section,
        label: tSettings(`sections.${section}`),
      }))}
    >
      {children}
    </TournamentSideNavSwitch>
  );
}
