import { useTranslations } from "next-intl";
import type { HTMLAttributes, ReactNode } from "react";

import { Section } from "@/shared/ui/section";
import { RESERVES_ID } from "../lib/focus";
import { FormationBlock } from "./formation-block";
import styles from "./reserves-block.module.scss";

type Props = Omit<HTMLAttributes<HTMLElement>, "children"> & {
  count: number;
  /** Um jogador arrastado pode ser solto aqui e está em cima. */
  highlight?: boolean;
  /** Para receber jogadores arrastados (`useDroppable`). */
  ref?: (node: HTMLElement | null) => void;
  /** Os reservas (`PlayerLine`), na ordem de inscrição. */
  children: ReactNode;
};

/**
 * Os reservas: quem está inscrito sem time, na ordem de inscrição (no TDM em
 * andamento, também quem foi substituído). Na edição, é onde se solta quem
 * sai de um time; recebe o foco depois disso.
 */
export function ReservesBlock({ count, highlight, children, ...rest }: Props) {
  const t = useTranslations("Team");
  return (
    <Section title={t("reserves.title", { count })}>
      <FormationBlock
        id={RESERVES_ID}
        tabIndex={-1}
        highlight={highlight}
        {...rest}
      >
        {count === 0 ? (
          <p className={styles.empty}>{t("reserves.empty")}</p>
        ) : (
          <ul>{children}</ul>
        )}
      </FormationBlock>
    </Section>
  );
}
