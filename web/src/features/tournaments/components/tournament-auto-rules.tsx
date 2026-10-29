import { Stack } from "@carbon/react";
import { useTranslations } from "next-intl";
import { Fragment, type ReactNode } from "react";

import type { components } from "@/shared/api/schema";
import { useFormatMinutes } from "@/shared/hooks/use-format-minutes";
import { LocalDateTime } from "@/shared/ui/local-date-time";
import { LocalTime } from "@/shared/ui/local-time";
import { Section } from "@/shared/ui/section";
import { bracketKind, mapsToWin, repeatScope } from "../lib/format";
import styles from "./tournament-auto-rules.module.scss";

type Props = {
  tournament: components["schemas"]["Tournament"];
  /** Quem vê ainda edita o torneio: a seção diz de onde o texto sai. */
  editable?: boolean;
};

/**
 * As regras que saem da configuração do torneio, para quem joga, na ordem em
 * que acontecem: inscrição, início e check-in, chave, séries, campeões, envio
 * e resposta do resultado e W.O. Frases curtas, na segunda pessoa; cada
 * trecho segue a configuração (sem check-in, eliminação simples, sem bans...)
 * e muda sozinho quando ela muda, porque nada é gravado. O que ela não diz
 * (mapas, conduta, queda de conexão) fica nas regras adicionais.
 */
export function TournamentAutoRules({ tournament, editable }: Props) {
  const t = useTranslations("Tournament.autoRules");
  const formatMinutes = useFormatMinutes();
  const [stage] = tournament.stages;
  const { bans_per_game: bans } = tournament.champion_rules;
  const { starts_at: startsAt, check_in_opens_at: opensAt } = tournament;
  const checkIn = tournament.check_in_minutes;
  const checkInWindow = checkIn == null ? "" : formatMinutes(checkIn);
  const finals = stage.finals_best_of;
  // Tudo de um mapa só: nada a dizer sobre repetir mapa nem sobre o seguinte.
  const severalMaps = Math.max(stage.best_of, finals) > 1;

  const paragraphs: [string, ReactNode[]][] = [
    [
      "registration",
      [
        t(checkIn == null ? "registrationNoCheckIn" : "registration", {
          max: tournament.max_participants,
        }),
      ],
    ],
    [
      "checkIn",
      [
        startsAt &&
          t.rich("start", { date: () => <LocalDateTime value={startsAt} /> }),
        checkIn == null
          ? t("noCheckIn")
          : startsAt && opensAt
            ? t.rich("checkInAt", {
                window: checkInWindow,
                time: () => <LocalTime value={opensAt} />,
              })
            : t("checkIn", { window: checkInWindow }),
        checkIn != null && t("checkInDo"),
      ],
    ],
    [
      "bracket",
      [t(`bracket.${stage.bracket}`), t(`bracketRule.${bracketKind(stage)}`)],
    ],
    [
      "series",
      [
        t("series", { bestOf: stage.best_of, wins: mapsToWin(stage.best_of) }),
        finals !== stage.best_of &&
          t(`finals.${stage.bracket}`, {
            bestOf: finals,
            wins: mapsToWin(finals),
          }),
        severalMaps && t("maps"),
      ],
    ],
    [
      "champions",
      [
        bans > 0 ? t("champions.bans", { count: bans }) : t("noBans"),
        t(`champions.repeat.${repeatScope(tournament.champion_rules)}`),
      ],
    ],
    ["report", [t("report"), severalMaps && t("nextMap")]],
    [
      "confirm",
      [t("confirm", { window: formatMinutes(tournament.contest_minutes) })],
    ],
    ["walkover", [t("walkover")]],
  ];

  return (
    <Section
      title={t("title")}
      description={editable ? t("generated") : undefined}
    >
      <Stack gap={5} className={styles.text}>
        {paragraphs.map(([key, sentences]) => (
          <p key={key}>
            {sentences.filter(Boolean).map((sentence, index) => (
              // biome-ignore lint/suspicious/noArrayIndexKey: frases fixas, na ordem.
              <Fragment key={index}>
                {index > 0 && " "}
                {sentence}
              </Fragment>
            ))}
          </p>
        ))}
      </Stack>
    </Section>
  );
}
