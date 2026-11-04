import { CheckmarkFilled, ErrorFilled } from "@carbon/icons-react";
import { type ReactNode, useId } from "react";

import { cn } from "@/shared/lib/cn";
import { Tooltip } from "@/shared/ui/tooltip";
import type { TileState } from "../lib/veto";
import styles from "./veto-tile.module.scss";

type Props = {
  /** `MapThumbnail` (16:9) ou `ChampionPortrait` (3:4), já com `out` no banido. */
  image: ReactNode;
  name: string;
  state: TileState;
  /**
   * O campeão: o retrato estreito, sem nome nem etiqueta à vista (só o banido
   * leva a marca), com a altura fixa. O nome fica para o leitor de tela.
   */
  portrait?: boolean;
  /** Etiqueta curta embaixo do nome ("Partida 2", "Voto de Ana"); só nos mapas. */
  note?: string;
  /**
   * O motivo de quem vê não poder escolher o campeão, na dica discreta do
   * tile (`state="unavailable"`): ele fica apagado, mas focável e lido como
   * botão desabilitado, para o motivo abrir também pelo teclado e pelo toque.
   */
  hint?: string;
  /** O estado por extenso para o leitor de tela ("Banido por Ana"), quando não é botão. */
  described?: string;
  /** Na vez de quem vê: o cartão inteiro vira o botão, com a ação escrita nele. */
  action?: {
    kind: "ban" | "pick";
    /** "Banir", "Escolher", "Votar". */
    text: string;
    /** Rótulo acessível com o nome ("Banir Awoken"). */
    label: string;
    /** Outra ação do veto está no ar: o botão fica, sem responder. */
    disabled?: boolean;
    onClick: () => void;
  };
};

/**
 * Um mapa ou um campeão no veto, na votação e no draft, como o mapa do pool
 * no formulário do torneio (um tile do Carbon): a imagem em cima e, embaixo,
 * o nome e a etiqueta, com o ícone de estado no canto de baixo (o das
 * notificações: check verde no escolhido, X vermelho no banido, com a imagem
 * em cinza; fora da imagem, onde a cor tem contraste garantido). O
 * escolhido ganha a borda de selecionado. Na vez de quem vê, o tile inteiro é
 * o botão (o tile clicável: fundo de hover, contorno de foco), com a ação no
 * pé, com a cara do botão terciário (o de perigo no ban), porque no celular
 * não há hover que a mostre; a etiqueta também descreve o botão para o leitor
 * de tela. Fora da vez, o disponível guarda o lugar da ação: a grade não anda
 * embaixo do dedo quando a vez chega.
 *
 * O campeão (`portrait`) só responde "posso escolher agora?": sem etiqueta
 * nem check de escolhido (quem escolheu o quê está no resumo do draft), o
 * banido com a marca e o que quem vê não pode escolher apagado, com o motivo
 * numa dica (`hint`). O pé tem sempre a altura da ação, com ela ou sem.
 */
export function VetoTile({
  image,
  name,
  state,
  portrait = false,
  note,
  hint,
  described,
  action,
}: Props) {
  const noteId = useId();
  const Icon =
    state === "banned"
      ? ErrorFilled
      : state === "picked" && !portrait
        ? CheckmarkFilled
        : null;
  const reserved = (
    <span aria-hidden className={cn(styles.action, styles.reserved)}>
      &nbsp;
    </span>
  );
  const foot = action ? (
    <span aria-hidden className={styles.action}>
      {action.text}
    </span>
  ) : portrait ? (
    <span className={styles.slot}>
      {reserved}
      {Icon && <Icon aria-hidden size={16} className={styles.icon} />}
    </span>
  ) : (
    state === "available" && reserved
  );
  const body = (
    <>
      <span className={styles.image}>{image}</span>
      <span className={styles.text}>
        <span className={portrait ? styles.srOnly : styles.name}>{name}</span>
        {!portrait && (note || Icon) && (
          <span className={styles.state}>
            <span id={noteId} className={styles.note}>
              {note}
            </span>
            {Icon && <Icon aria-hidden size={16} className={styles.icon} />}
          </span>
        )}
        {foot}
      </span>
    </>
  );
  const disabled = !action && hint != null;
  const surface = action ? (
    <button
      type="button"
      className={cn(styles.surface, styles.button, styles[action.kind])}
      aria-label={action.label}
      aria-describedby={note ? noteId : undefined}
      disabled={action.disabled}
      onClick={action.onClick}
    >
      {body}
    </button>
  ) : disabled ? (
    // biome-ignore lint/a11y/useSemanticElements: o `button` desabilitado sairia da ordem do teclado, e a dica com o motivo precisa abrir no foco e no toque.
    <div
      role="button"
      tabIndex={0}
      aria-disabled="true"
      aria-label={name}
      className={styles.surface}
    >
      {body}
    </div>
  ) : (
    <div className={styles.surface}>
      {body}
      {described && <span className={styles.srOnly}>{described}</span>}
    </div>
  );
  return (
    <li className={cn(styles.tile, styles[state], portrait && styles.portrait)}>
      {disabled ? (
        <Tooltip
          description={hint}
          highContrast={false}
          autoAlign
          className={styles.tip}
        >
          {surface}
        </Tooltip>
      ) : (
        surface
      )}
    </li>
  );
}
