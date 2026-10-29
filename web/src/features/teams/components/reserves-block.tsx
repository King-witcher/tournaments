import { useTranslations } from "next-intl";
import type { HTMLAttributes, ReactNode } from "react";

import { ErrorText } from "@/shared/ui/error-text";
import { Section } from "@/shared/ui/section";
import { Spinner } from "@/shared/ui/spinner";
import { RESERVES_ID } from "../lib/focus";
import { FormationBlock } from "./formation-block";
import styles from "./reserves-block.module.scss";

type Props = Omit<HTMLAttributes<HTMLElement>, "children"> & {
  count: number;
  /** Um jogador arrastado pode ser solto aqui e está em cima. */
  highlight?: boolean;
  /** Mandar alguém para a reserva está em andamento. */
  busy?: boolean;
  /** O erro da última ação aqui, já traduzido. */
  error?: string;
  /** Os reservas (`PlayerLine`), na ordem de inscrição. */
  children: ReactNode;
};

/**
 * Os reservas: quem está inscrito sem time, na ordem de inscrição (no TDM em
 * andamento, também quem foi substituído). Na edição, é onde se solta quem
 * sai de um time; recebe o foco depois disso.
 */
export function ReservesBlock({
  count,
  highlight,
  busy,
  error,
  children,
  ...rest
}: Props) {
  const t = useTranslations("Team");
  return (
    <Section
      title={t("reserves.title", { count })}
      description={t("reserves.what")}
    >
      <FormationBlock
        id={RESERVES_ID}
        tabIndex={-1}
        aria-busy={busy || undefined}
        highlight={highlight}
        {...rest}
      >
        {busy && <Spinner label={t("saving")} />}
        {count === 0 ? (
          <p className={styles.empty}>{t("reserves.empty")}</p>
        ) : (
          <ul>{children}</ul>
        )}
        {error && <ErrorText>{error}</ErrorText>}
      </FormationBlock>
    </Section>
  );
}
