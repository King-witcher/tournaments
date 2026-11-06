import { CoverSection } from "@/features/images/components/cover-section";
import { getImageSettings } from "@/features/images/server";
import { loadMaps } from "@/features/matches/server";
import { getScreenshotSettings } from "@/features/screenshots/server";
import { apiErrorMessage } from "@/shared/api/errors";
import { Column, Grid } from "@/shared/ui/carbon/grid";
import { Stack } from "@/shared/ui/carbon/stack";
import { Notice } from "@/shared/ui/notice";
import { loadTournament } from "../server";
import { SettingsChampionsSection } from "./settings-champions-section";
import { SettingsFormatSection } from "./settings-format-section";
import { SettingsGeneralSection } from "./settings-general-section";
import { SettingsLanguagesSection } from "./settings-languages-section";
import { SettingsMapsSection } from "./settings-maps-section";
import { SettingsRegistrationSection } from "./settings-registration-section";
import { SettingsResultsSection } from "./settings-results-section";
import { SettingsTeamsSection } from "./settings-teams-section";
import { TournamentSettingsProvider } from "./tournament-settings-provider";

type Props = { tenant: string; slug: string };

/**
 * A aba Configurações do torneio, para organizer e owner: uma seção por
 * assunto, cada uma com o próprio Salvar (Geral, Formato, Times no TDM,
 * Mapas, Campeões, Inscrições e prazos, Resultados, Outros idiomas), na
 * coluna estreita das configurações da comunidade. As regras adicionais ficam
 * na aba Regras. Para quem joga, o aviso de que a aba não é dele. Fica em
 * `<Suspense>`.
 */
export async function TournamentSettings({ tenant, slug }: Props) {
  const [loaded, maps, screenshots] = await Promise.all([
    loadTournament(tenant, slug),
    loadMaps(),
    getScreenshotSettings(),
    // A capa pede o mesmo, na mesma requisição: chega pronto.
    getImageSettings(),
  ]);
  if (loaded.kind !== "ok") return null;
  if (loaded.me.role === "player") {
    return (
      <Notice kind="error">{await apiErrorMessage("tenant.forbidden")}</Notice>
    );
  }

  const { tournament } = loaded;
  return (
    <Grid>
      <Column sm={4} md={8} lg={8}>
        <TournamentSettingsProvider tenant={tenant} tournament={tournament}>
          <Stack gap={9}>
            <SettingsGeneralSection
              cover={
                <CoverSection
                  tenant={tenant}
                  tournament={tournament.slug}
                  url={tournament.cover_url}
                />
              }
            />
            <SettingsFormatSection />
            {tournament.mode === "tdm" && <SettingsTeamsSection />}
            <SettingsMapsSection maps={maps} />
            <SettingsChampionsSection />
            <SettingsRegistrationSection />
            <SettingsResultsSection
              screenshots={screenshots?.enabled ?? false}
            />
            <SettingsLanguagesSection />
          </Stack>
        </TournamentSettingsProvider>
      </Column>
    </Grid>
  );
}
