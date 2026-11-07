import { getLocale, getTranslations } from "next-intl/server";

import { rulesImagesOf } from "@/features/images/lib/rules-images";
import { getImageSettings } from "@/features/images/server";
import { loadMaps } from "@/features/matches/server";
import { Stack } from "@/shared/ui/carbon/stack";
import { MarkdownText } from "@/shared/ui/markdown-text";
import { Section } from "@/shared/ui/section";
import { rulesSource } from "../lib/localize";
import { isFinished } from "../lib/status";
import { loadTournament } from "../server";
import { RulesEditor } from "./rules-editor";
import { TournamentAutoRules } from "./tournament-auto-rules";
import styles from "./tournament-rules.module.scss";

type Props = { tenant: string; slug: string };

/**
 * A seção Regras, na largura de leitura: primeiro "Como funciona", o texto
 * que sai da configuração do torneio; embaixo, as regras que a organização
 * escreve (em
 * Markdown, `MarkdownText`), no idioma de quem vê (sem tradução, o
 * original). Para a organização, até o torneio encerrar, a seção das regras
 * da organização aparece sempre, com "Editar" (ou o convite a escrever) ali
 * mesmo; para os outros, só quando há texto: sem ele, o automático basta.
 * Fica em `<Suspense>`.
 */
export async function TournamentRules({ tenant, slug }: Props) {
  const [t, locale, loaded, maps, imageSettings] = await Promise.all([
    getTranslations("Tournament"),
    getLocale(),
    loadTournament(tenant, slug),
    loadMaps(),
    getImageSettings(),
  ]);
  if (loaded.kind !== "ok") return null;

  const { me, tournament } = loaded;
  const source = rulesSource(tournament, locale);
  const editable = me.role !== "player" && !isFinished(tournament.status);
  const images = rulesImagesOf(imageSettings);
  return (
    // Texto corrido e o campo das regras na largura de leitura.
    <div className={styles.reading}>
      <Stack gap={7}>
        <TournamentAutoRules
          tournament={tournament}
          maps={maps}
          editable={editable}
        />
        {(editable || source.rules) && (
          <Section title={t("extraRules")}>
            {editable ? (
              <RulesEditor
                tenant={tenant}
                tournament={tournament.slug}
                locale={source.locale}
                rules={source.rules}
                translation={source.translation}
                images={images}
              />
            ) : (
              <MarkdownText imagePrefix={images?.prefix}>
                {source.rules}
              </MarkdownText>
            )}
          </Section>
        )}
      </Stack>
    </div>
  );
}
