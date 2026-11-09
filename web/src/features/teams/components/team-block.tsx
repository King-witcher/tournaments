import { useTranslations } from "next-intl";
import type { HTMLAttributes, ReactNode } from "react";

import type { components } from "@/shared/api/schema";
import { useTeamName } from "../hooks/use-team-name";
import { teamBlockId } from "../lib/focus";
import { EmptySlot } from "./empty-slot";
import { FormationBlock } from "./formation-block";
import styles from "./team-block.module.scss";

type Props = Omit<HTMLAttributes<HTMLElement>, "children"> & {
  team: components["schemas"]["Team"];
  /** O seed antes do nome (depois de iniciar, a ordem da chave). */
  showSeed?: boolean;
  /** O estado do time (`TeamState`), que só aparece quando há o que dizer. */
  state?: ReactNode;
  /** Renomear, o ⋯ da organização. */
  actions?: ReactNode;
  /** Quantas vagas ainda faltam preencher: o card já vem no tamanho final. */
  emptySlots?: number;
  /** Na edição, as vagas alinham com o avatar, depois da alça. */
  indentSlots?: boolean;
  /** Um jogador arrastado pode ser solto aqui e está em cima. */
  highlight?: boolean;
  /** Para receber jogadores arrastados (`useDroppable`). */
  ref?: (node: HTMLElement | null) => void;
  /** Os jogadores (`PlayerLine`), o capitão primeiro. */
  children: ReactNode;
};

/**
 * Um time na formação: o nome (ou "Time de {capitão}"), o estado, as ações,
 * os jogadores e as vagas livres. Tudo numa linha por vaga, com o nome longo
 * em reticências: todos os cards ficam da mesma altura. Recebe o foco
 * (`tabIndex={-1}`) depois de uma ação que tira o controle de onde estava
 * (mover alguém para cá).
 */
export function TeamBlock({
  team,
  showSeed,
  state,
  actions,
  emptySlots = 0,
  indentSlots,
  highlight,
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
      highlight={highlight}
      {...rest}
    >
      <div className={styles.header}>
        <h3 id={`${id}-name`} className={styles.name} title={name}>
          {showSeed && team.seed != null && (
            <span className={styles.seed}>
              {t("seed", { seed: team.seed })}
            </span>
          )}
          {name}
        </h3>
        {state && <span className={styles.state}>{state}</span>}
        {actions && <span className={styles.actions}>{actions}</span>}
      </div>
      <ul>
        {children}
        {Array.from({ length: emptySlots }, (_, slot) => (
          // As vagas são iguais: nada as distingue além da posição.
          // biome-ignore lint/suspicious/noArrayIndexKey: ver acima
          <EmptySlot key={slot} indented={indentSlots} />
        ))}
      </ul>
    </FormationBlock>
  );
}
