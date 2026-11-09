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
  | "rules"
  | "settings";

export type TournamentTab = { key: TournamentTabKey; href: string };

/**
 * As seções do torneio, uma coisa por seção: Visão geral, Chave (a partir do
 * início), Classificação (quando `standings`), Inscritos, Regras (sempre),
 * Configurações (só para a organização). O que não se aplica
 * não entra na lista.
 */
export function tournamentTabs(
  base: string,
  {
    status,
    standings,
    organizer,
  }: { status: Status; standings: boolean; organizer: boolean },
): TournamentTab[] {
  const tab = (key: TournamentTabKey, path = `/${key}`) => ({
    key,
    href: `${base}${path}`,
  });
  return [
    tab("overview", ""),
    ...(hasStarted(status) ? [tab("bracket")] : []),
    ...(standings ? [tab("standings")] : []),
    tab("participants"),
    tab("rules"),
    ...(organizer ? [tab("settings")] : []),
  ];
}
