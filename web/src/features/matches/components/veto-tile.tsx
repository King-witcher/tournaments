import { CheckmarkFilled, Locked } from "@carbon/icons-react";
import type { ReactNode } from "react";

import { cn } from "@/shared/lib/cn";
import type { TileState } from "../lib/veto";
import styles from "./veto-tile.module.scss";

type Props = {
  /** `MapThumbnail` (16:9) ou `ChampionPortrait` (3:4), já com `out` no banido. */
  image: ReactNode;
  name: string;
  state: TileState;
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
    onClick: () => void;
  };
};

/**
 * Um mapa ou um campeão no veto, na votação e no draft: a imagem com a borda
 * do estado (banido: vermelha, imagem em cinza e cadeado; escolhido: verde e
 * check), o nome, a etiqueta e, na vez de quem vê, a ação escrita no pé do
 * cartão, que inteiro é o botão (no celular não há hover que mostre).
 */
export function VetoTile({
  image,
  name,
  state,
  note,
  described,
  action,
}: Props) {
  const icon =
    state === "banned" ? (
      <Locked aria-hidden size={16} />
    ) : state === "picked" ? (
      <CheckmarkFilled aria-hidden size={16} />
    ) : null;
  const body = (
    <>
      <span className={styles.frame}>
        {image}
        {icon && <span className={styles.icon}>{icon}</span>}
      </span>
      <span className={styles.name}>{name}</span>
      {note && <span className={styles.note}>{note}</span>}
    </>
  );
  return (
    <li className={cn(styles.tile, styles[state])}>
      {action ? (
        <button
          type="button"
          className={cn(styles.button, styles[action.kind])}
          aria-label={action.label}
          onClick={action.onClick}
        >
          {body}
          <span aria-hidden className={styles.action}>
            {action.text}
          </span>
        </button>
      ) : (
        <div className={styles.body}>
          {body}
          {described && <span className={styles.srOnly}>{described}</span>}
        </div>
      )}
    </li>
  );
}
