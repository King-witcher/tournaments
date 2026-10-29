import { Stack } from "@carbon/react";
import { useTranslations } from "next-intl";
import { Fragment, type ReactNode } from "react";

import type { components } from "@/shared/api/schema";
import { useFormatMinutes } from "@/shared/hooks/use-format-minutes";
import { LocalDateTime } from "@/shared/ui/local-date-time";
import { LocalTime } from "@/shared/ui/local-time";
import { Section } from "@/shared/ui/section";
import { useRuleSentences } from "../hooks/use-rule-sentences";
import { finalsBestOf, mapsToWin } from "../lib/format";
import styles from "./tournament-auto-rules.module.scss";

type Props = {
  tournament: components["schemas"]["Tournament"];
  /** Quem vê ainda edita o torneio: a seção diz de onde o texto sai. */
  editable?: boolean;
};

/**
 * As regras que saem da configuração do torneio, para quem joga, na ordem em
 * que acontecem: inscrição, início e check-in, (no TDM) os times, chave (ou
 * round robin, suíço, com a tabela e os desempates), séries, campeões, envio
 * e resposta do resultado, W.O. e (no TDM) substituição e desfalque. Frases
 * curtas, na segunda pessoa; cada trecho segue a configuração (sem check-in,
 * eliminação simples, sem bans, duelo ou TDM...) e muda sozinho quando ela
 * muda, porque nada é gravado. O que ela não diz (mapas, conduta, queda de
 * conexão) fica nas regras adicionais.
 */
export function TournamentAutoRules({ tournament, editable }: Props) {
  const t = useTranslations("Tournament.autoRules");
  const formatMinutes = useFormatMinutes();
  const rules = useRuleSentences();
  const tdm = tournament.mode === "tdm";
  const [stage] = tournament.stages;
  const { starts_at: startsAt, check_in_opens_at: opensAt } = tournament;
  const checkIn = tournament.check_in_minutes;
  const checkInWindow = checkIn == null ? "" : formatMinutes(checkIn);
  const finals = finalsBestOf(stage) ?? stage.best_of;
  // Tudo de um mapa só: nada a dizer sobre repetir mapa nem sobre o seguinte.
  const severalMaps = Math.max(stage.best_of, finals) > 1;
  // Round robin ou suíço: a tabela, os frags no desempate, o W.O. em mapas.
  const table = stage.format === "elimination" ? null : stage.format;
  const wins = mapsToWin(stage.best_of);
  // No TDM, as rodadas automáticas do suíço contam times, não inscritos.
  const bracket =
    stage.format === "elimination"
      ? t(`bracket.${stage.bracket}`)
      : stage.format === "swiss"
        ? stage.rounds != null
          ? t(tdm ? "tdm.swissRounds" : "bracket.swissRounds", {
              rounds: stage.rounds,
            })
          : t(tdm ? "tdm.swiss" : "bracket.swiss")
        : t("bracket.round_robin");

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
          ? t(tdm ? "tdm.noCheckIn" : "noCheckIn")
          : startsAt && opensAt
            ? t.rich("checkInAt", {
                window: checkInWindow,
                time: () => <LocalTime value={opensAt} />,
              })
            : t("checkIn", { window: checkInWindow }),
        checkIn != null && t(tdm ? "tdm.checkInDo" : "checkInDo"),
      ],
    ],
    ["teams", rules.teams(tournament)],
    ["bracket", [bracket, rules.bracketRule(tournament)]],
    ["table", table ? [t(`tableRule.${table}`)] : []],
    [
      "series",
      [
        t("series", { bestOf: stage.best_of, wins: mapsToWin(stage.best_of) }),
        stage.format === "elimination" &&
          finals !== stage.best_of &&
          t(`finals.${stage.bracket}`, {
            bestOf: finals,
            wins: mapsToWin(finals),
          }),
        severalMaps && t("maps"),
      ],
    ],
    ["champions", rules.champions(tournament)],
    [
      "report",
      [
        t(tdm ? "tdm.report" : "report"),
        severalMaps && t("nextMap"),
        table && t("fragsTable"),
      ],
    ],
    [
      "confirm",
      [
        t(tdm ? "tdm.confirm" : "confirm", {
          window: formatMinutes(tournament.contest_minutes),
        }),
      ],
    ],
    [
      "walkover",
      [
        t(tdm ? "tdm.walkover" : "walkover"),
        table && t("walkoverTable", { wins }),
        table && t(`withdrawal.${table}`),
      ],
    ],
    [
      "lineup",
      tdm
        ? [
            t("tdm.substitution"),
            t("tdm.shortHanded"),
            tournament.counts_for_rating && t("tdm.rating"),
          ]
        : [],
    ],
  ];

  return (
    <Section
      title={t("title")}
      description={editable ? t("generated") : undefined}
    >
      <Stack gap={5} className={styles.text}>
        {paragraphs
          .filter(([, sentences]) => sentences.some(Boolean))
          .map(([key, sentences]) => (
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
