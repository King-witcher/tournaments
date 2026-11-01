import { CheckmarkFilled, ErrorFilled } from "@carbon/icons-react";
import { type ReactNode, useId } from "react";

import { cn } from "@/shared/lib/cn";
import type { TileState } from "../lib/veto";
import styles from "./veto-tile.module.scss";

type Props = {
  /** `MapThumbnail` (16:9) ou `ChampionPortrait` (3:4), já com `out` no banido. */
  image: ReactNode;
  name: string;
  state: TileState;
  /** `narrow`: o retrato do campeão, estreito no celular (texto menor). */
  size?: "narrow";
  /** Etiqueta curta embaixo do nome ("Partida 2", "Voto de Ana", "Já usado"). */
  note?: string;
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
 * o botão (o tile clicável: fundo de hover, contorno de foco), com a ação
 * escrita no pé, porque no celular não há hover que a mostre; a etiqueta
 * também descreve o botão para o leitor de tela.
 */
export function VetoTile({
  image,
  name,
  state,
  size,
  note,
  described,
  action,
}: Props) {
  const noteId = useId();
  const Icon =
    state === "banned"
      ? ErrorFilled
      : state === "picked"
        ? CheckmarkFilled
        : null;
  const body = (
    <>
      <span className={styles.image}>{image}</span>
      <span className={styles.text}>
        <span className={styles.name}>{name}</span>
        {(note || Icon) && (
          <span className={styles.state}>
            <span id={noteId} className={styles.note}>
              {note}
            </span>
            {Icon && <Icon aria-hidden size={16} className={styles.icon} />}
          </span>
        )}
        {action && (
          <span aria-hidden className={styles.action}>
            {action.text}
          </span>
        )}
      </span>
    </>
  );
  return (
    <li className={cn(styles.tile, styles[state], size && styles[size])}>
      {action ? (
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
      ) : (
        <div className={styles.surface}>
          {body}
          {described && <span className={styles.srOnly}>{described}</span>}
        </div>
      )}
    </li>
  );
}
