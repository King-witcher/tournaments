import type { components } from "@/shared/api/schema";
import type { SettingsSection } from "./settings-sections";
import { hasStarted } from "./status";

/**
 * O endereço de uma seção das Configurações do torneio (`base` é o do
 * torneio): Geral é o das Configurações; as outras, um nível abaixo
 * (`/settings/maps`).
 */
export function settingsHref(base: string, section: SettingsSection) {
  return section === "general"
    ? `${base}/settings`
    : `${base}/settings/${section}`;
}

type Status = components["schemas"]["TournamentStatus"];

export type TournamentTabKey =
  | "overview"
  | "bracket"
  | "standings"
  | "participants"
  | "teams"
  | "seeding"
  | "rules"
  | "settings";

export type TournamentTab = { key: TournamentTabKey; href: string };

/**
 * As seções do torneio, uma coisa por seção: Visão geral, Chave (a partir do
 * início), Classificação (quando `standings`), Inscritos, Times (no torneio
 * de times, para todos) e Ordem da chave (só para a organização; depois de
 * iniciar, para consulta), as três depois de publicar, porque o rascunho não
 * tem inscritos; Regras (sempre), Configurações (só para a organização). O
 * que não se aplica não entra na lista.
 */
export function tournamentTabs(
  base: string,
  {
    status,
    standings,
    organizer,
    teams,
  }: {
    status: Status;
    standings: boolean;
    organizer: boolean;
    teams: boolean;
  },
): TournamentTab[] {
  const tab = (key: TournamentTabKey, path = `/${key}`) => ({
    key,
    href: `${base}${path}`,
  });
  return [
    tab("overview", ""),
    ...(hasStarted(status) ? [tab("bracket")] : []),
    ...(standings ? [tab("standings")] : []),
    ...(status === "draft"
      ? []
      : [
          tab("participants"),
          ...(teams ? [tab("teams")] : []),
          ...(organizer ? [tab("seeding")] : []),
        ]),
    tab("rules"),
    ...(organizer ? [tab("settings")] : []),
  ];
}
