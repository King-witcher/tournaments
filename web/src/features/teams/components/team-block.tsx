import { useTranslations } from "next-intl";
import type { HTMLAttributes, ReactNode } from "react";

import type { components } from "@/shared/api/schema";
import { ErrorText } from "@/shared/ui/error-text";
import { Spinner } from "@/shared/ui/spinner";
import { useTeamName } from "../hooks/use-team-name";
import { teamBlockId } from "../lib/focus";
import { FormationBlock } from "./formation-block";
import styles from "./team-block.module.scss";

type Props = Omit<HTMLAttributes<HTMLElement>, "children"> & {
  team: components["schemas"]["Team"];
  /** O seed antes do nome (depois de iniciar, a ordem da chave). */
  showSeed?: boolean;
  /** O estado do time (`TeamStateTag`). */
  tag?: ReactNode;
  /** Renomear, o ⋯ da organização. */
  actions?: ReactNode;
  /** Um jogador arrastado pode ser solto aqui e está em cima. */
  highlight?: boolean;
  /** Uma ação neste time está em andamento (soltar alguém nele). */
  busy?: boolean;
  /** O erro da última ação neste time, já traduzido. */
  error?: string;
  /** Os jogadores (`PlayerLine`), o capitão primeiro. */
  children: ReactNode;
};

/**
 * Um time na formação: o nome (ou "Time de {capitão}"), o estado, as ações
 * e os jogadores. Recebe o foco (`tabIndex={-1}`) depois de uma ação que
 * tira o controle de onde estava (mover alguém para cá).
 */
export function TeamBlock({
  team,
  showSeed,
  tag,
  actions,
  highlight,
  busy,
  error,
  children,
  ...rest
}: Props) {
  const t = useTranslations("Team");
  const name = useTeamName()(team);
  const id = teamBlockId(team.id);

  return (
    <FormationBlock
      as="section"
      id={id}
      tabIndex={-1}
      aria-labelledby={`${id}-name`}
      aria-busy={busy || undefined}
      highlight={highlight}
      {...rest}
    >
      <div className={styles.header}>
        <h3 id={`${id}-name`} className={styles.name}>
          {showSeed && team.seed != null && (
            <span className={styles.seed}>
              {t("seed", { seed: team.seed })}
            </span>
          )}
          {name}
        </h3>
        {tag}
        {(actions || busy) && (
          <span className={styles.actions}>
            {busy && <Spinner label={t("saving")} />}
            {actions}
          </span>
        )}
      </div>
      <ul>{children}</ul>
      {error && <ErrorText>{error}</ErrorText>}
    </FormationBlock>
  );
}
