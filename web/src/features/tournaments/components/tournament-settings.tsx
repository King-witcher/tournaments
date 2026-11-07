import { CoverSection } from "@/features/images/components/cover-section";
import { rulesImagesOf } from "@/features/images/lib/rules-images";
import { getImageSettings } from "@/features/images/server";
import { loadMaps } from "@/features/matches/server";
import { getScreenshotSettings } from "@/features/screenshots/server";
import { apiErrorMessage } from "@/shared/api/errors";
import { Accordion } from "@/shared/ui/carbon/accordion";
import { Column, Grid } from "@/shared/ui/carbon/grid";
import { Notice } from "@/shared/ui/notice";
import { loadTournament } from "../server";
import { SettingsChampionsSection } from "./settings-champions-section";
import { SettingsFormatSection } from "./settings-format-section";
import { SettingsGeneralSection } from "./settings-general-section";
import { SettingsLanguagesSection } from "./settings-languages-section";
import { SettingsMapsSection } from "./settings-maps-section";
import { SettingsRegistrationSection } from "./settings-registration-section";
import { SettingsResultsSection } from "./settings-results-section";
import { TournamentSettingsProvider } from "./tournament-settings-provider";

type Props = { tenant: string; slug: string };

/**
 * A aba Configurações do torneio, para organizer e owner: um acordeão com uma
 * seção por item (Geral, Início e inscrições, Formato, Mapas, Campeões,
 * Resultados, Outros idiomas), cada uma com o resumo do que está salvo no
 * título e o próprio Salvar dentro, na coluna estreita das configurações da
 * comunidade. Fechada, a aba é o resumo da configuração inteira; vários itens
 * podem ficar abertos. As regras da organização ficam na aba Regras. Para quem
 * joga, o aviso de que a aba não é dele. Fica em `<Suspense>`.
 */
export async function TournamentSettings({ tenant, slug }: Props) {
  const [loaded, maps, screenshots, imageSettings] = await Promise.all([
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
          <Accordion>
            <SettingsGeneralSection
              cover={
                <CoverSection
                  tenant={tenant}
                  tournament={tournament.slug}
                  url={tournament.cover_url}
                />
              }
            />
            <SettingsRegistrationSection />
            <SettingsFormatSection />
            <SettingsMapsSection maps={maps} />
            <SettingsChampionsSection />
            <SettingsResultsSection
              screenshots={screenshots?.enabled ?? false}
            />
            <SettingsLanguagesSection
              rulesImages={rulesImagesOf(imageSettings)}
            />
          </Accordion>
        </TournamentSettingsProvider>
      </Column>
    </Grid>
  );
}
