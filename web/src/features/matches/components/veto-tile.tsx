import { CheckmarkFilled, ErrorFilled } from "@carbon/icons-react";
import { memo, useId } from "react";

import { ChampionPortrait } from "@/features/champions/components/champion-portrait";
import { MapThumbnail } from "@/features/maps/components/map-thumbnail";
import { cn } from "@/shared/lib/cn";
import { Tooltip } from "@/shared/ui/tooltip";
import type { TileState } from "../lib/veto";
import styles from "./veto-tile.module.scss";

type Props = {
  /**
   * O mapa ou o campeão: dá a imagem (`MapThumbnail` 16:9 ou, com
   * `portrait`, `ChampionPortrait` 3:4; em cinza no banido) e vai no
   * `onAction`.
   */
  slug: string;
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
  action?: "ban" | "pick";
  /** "Banir", "Escolher", "Votar". */
  actionText?: string;
  /** Rótulo acessível com o nome ("Banir Awoken"). */
  actionLabel?: string;
  /** Outra ação do veto está no ar: o botão fica, sem responder. */
  disabled?: boolean;
  /**
   * O clique na ação, com o `slug`: a mesma função para todos os tiles do
   * tabuleiro, que entre renderizações não muda (os tiles só renderizam de
   * novo quando o que mostram muda).
   */
  onAction?: (slug: string) => void;
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
 *
 * Só com props simples (`memo`): o clique no mapa renderiza o tabuleiro duas
 * ou três vezes (o passo marcado, a resposta, a recarga), e os tiles que não
 * mudaram ficam como estão, com as dicas e as imagens.
 */
export const VetoTile = memo(function VetoTile({
  slug,
  name,
  state,
  portrait = false,
  note,
  hint,
  described,
  action,
  actionText,
  actionLabel,
  disabled: busy,
  onAction,
}: Props) {
  const noteId = useId();
  const out = state === "banned";
  const image = portrait ? (
    <ChampionPortrait slug={slug} out={out} />
  ) : (
    <MapThumbnail slug={slug} out={out} />
  );
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
      {actionText}
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
      className={cn(styles.surface, styles.button, styles[action])}
      aria-label={actionLabel}
      aria-describedby={note ? noteId : undefined}
      disabled={busy}
      onClick={() => onAction?.(slug)}
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
});
