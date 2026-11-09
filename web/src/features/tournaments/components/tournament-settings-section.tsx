import { rulesImagesOf } from "@/features/images/lib/rules-images";
import { getImageSettings } from "@/features/images/server";
import { loadMaps } from "@/features/matches/server";
import { getScreenshotSettings } from "@/features/screenshots/server";
import type { SettingsSection } from "../lib/settings-sections";
import { loadTournament } from "../server";
import { SettingsChampionsSection } from "./settings-champions-section";
import { SettingsFormatSection } from "./settings-format-section";
import { SettingsGeneralSection } from "./settings-general-section";
import { SettingsLanguagesSection } from "./settings-languages-section";
import { SettingsMapsSection } from "./settings-maps-section";
import { SettingsRegistrationSection } from "./settings-registration-section";
import { SettingsResultsSection } from "./settings-results-section";
import { TournamentInviteSettings } from "./tournament-invite-settings";

type Props = { tenant: string; slug: string; section: SettingsSection };

/**
 * A página de uma seção das Configurações, dentro de `TournamentSettings`
 * (que tem o estado delas): busca só o que a seção usa além do torneio (os
 * mapas, as capturas, as imagens das regras) e a monta. Para quem joga, nada:
 * o layout já mostra o aviso. Fica em `<Suspense>`.
 */
export async function TournamentSettingsSection({
  tenant,
  slug,
  section,
}: Props) {
  const loaded = await loadTournament(tenant, slug);
  if (loaded.kind !== "ok" || loaded.me.role === "player") return null;

  switch (section) {
    case "general": {
      const images = await getImageSettings();
      return (
        <SettingsGeneralSection
          coverMaxBytes={images?.enabled ? images.max_bytes : undefined}
        />
      );
    }
    case "registration":
      return (
        <SettingsRegistrationSection
          invite={
            <TournamentInviteSettings
              tenant={tenant}
              tournament={loaded.tournament}
              deletionScheduled={loaded.me.deletion_scheduled_for != null}
            />
          }
        />
      );
    case "format":
      return <SettingsFormatSection />;
    case "maps":
      return <SettingsMapsSection maps={await loadMaps()} />;
    case "champions":
      return <SettingsChampionsSection />;
    case "results": {
      const screenshots = await getScreenshotSettings();
      return (
        <SettingsResultsSection screenshots={screenshots?.enabled ?? false} />
      );
    }
    case "languages":
      return (
        <SettingsLanguagesSection
          rulesImages={rulesImagesOf(await getImageSettings())}
        />
      );
  }
}
