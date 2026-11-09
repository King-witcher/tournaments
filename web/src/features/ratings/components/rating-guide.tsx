import { getTranslations } from "next-intl/server";

import { ListItem, UnorderedList } from "@/shared/ui/carbon/list";
import { Stack } from "@/shared/ui/carbon/stack";
import { Panel } from "@/shared/ui/panel";
import { Section } from "@/shared/ui/section";
import { RatingGuideFormulas } from "./rating-guide-formulas";
import { RatingGuideNewPlayers } from "./rating-guide-new-players";
import { RatingGuidePoints } from "./rating-guide-points";

const COUNTS = [
  "each",
  "winner",
  "tournaments",
  "walkover",
  "ranking",
  "inactive",
] as const;

/**
 * O artigo do rating: em palavras e com exemplos para quem joga, e as
 * fórmulas no fim para quem quer a conta. Texto estático, sem dado da
 * comunidade: fala dos valores padrão e avisa que cada uma pode mudá-los.
 */
export async function RatingGuide() {
  const t = await getTranslations("RatingGuide");
  return (
    <Panel>
      <Stack gap={7}>
        <Stack gap={5}>
          <p>{t("intro.what")}</p>
          <p>{t("intro.defaults")}</p>
        </Stack>
        <RatingGuidePoints />
        <RatingGuideNewPlayers />
        <Section title={t("counts.title")}>
          {/* `isExpressive`: o corpo do `body-02`, igual aos parágrafos. */}
          <UnorderedList isExpressive>
            {COUNTS.map((key) => (
              <ListItem key={key}>{t(`counts.${key}`)}</ListItem>
            ))}
          </UnorderedList>
        </Section>
        <Section title={t("teams.title")}>
          <p>{t("teams.text")}</p>
          <p>{t("teams.example")}</p>
        </Section>
        <Section title={t("changes.title")}>
          <p>{t("changes.recompute")}</p>
          <p>{t("changes.manual")}</p>
        </Section>
        <RatingGuideFormulas />
      </Stack>
    </Panel>
  );
}
