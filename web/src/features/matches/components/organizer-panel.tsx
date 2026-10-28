import { AccordionItem, Stack } from "@carbon/react";
import { useTranslations } from "next-intl";

import type { MatchTarget } from "../lib/match-target";
import styles from "./organizer-panel.module.scss";
import { WalkoverForm } from "./walkover-form";

type Props = {
  target: MatchTarget;
  names: { a: string; b: string };
};

/**
 * Para organizer e owner, numa partida pronta ou decidida: o bloco fechado
 * "Organização" com o W.O. na série inteira (é exceção). Confirmar agora,
 * resolver, corrigir e anular um mapa ficam na linha de cada mapa. Vai
 * dentro de um `Accordion`; quem monta troca a `key` a cada resultado, para
 * o bloco fechar depois de salvar.
 */
export function OrganizerPanel({ target, names }: Props) {
  const t = useTranslations("Match.organizer");

  return (
    <AccordionItem title={t("title")}>
      <Stack gap={5}>
        <p className={styles.hint}>{t("hint")}</p>
        <WalkoverForm target={target} names={names} />
      </Stack>
    </AccordionItem>
  );
}
