import { cn } from "@/shared/lib/cn";
import styles from "./bracket-diagram.module.scss";

/** As cinco escolhas do Chaveamento (as duas eliminações separadas). */
export type BracketDiagramFormat =
  | "double"
  | "single"
  | "round_robin"
  | "swiss"
  | "placement";

/**
 * Os traços de cada formato, num quadro de 320 × 80 que estica na largura do
 * tile (o traço não estica: fica em 1px):
 * - simples: oito entram, a cada rodada passa a metade, um vence;
 * - dupla: a chave de cima e, embaixo, a repescagem, mais longa (recebe quem
 *   cai de cima a cada rodada), as duas se juntando na grande final;
 * - todos contra todos: cinco em fila, cada um ligado a todos os outros por
 *   um arco;
 * - suíço: os grupos de pontuação, que se abrem a cada rodada (quem vence
 *   sobe, quem perde desce);
 * - disputa de posições: pares soltos, que não se juntam.
 */
const PATHS: Record<BracketDiagramFormat, string> = {
  single:
    "M1 5H80V15H1M1 25H80V35H1M1 45H80V55H1M1 65H80V75H1M80 10H160V30H80M80 50H160V70H80M160 20H240V60H160M240 40H319",
  double:
    "M1 4H64V12H1M1 20H64V28H1M64 8H128V24H64M128 16H224M1 44H64V52H1M64 48H128V60H64M128 54H176V72H128M176 63H224M224 16V63M224 39.5H319",
  round_robin:
    "M40 74Q70 42 100 74M40 74Q100 9 160 74M40 74Q130 -23 220 74M40 74Q160 -56 280 74M100 74Q130 42 160 74M100 74Q160 9 220 74M100 74Q190 -23 280 74M160 74Q190 42 220 74M160 74Q220 9 280 74M220 74Q250 42 280 74",
  swiss:
    "M8 40L109 30M8 40L109 50M109 30L210 20M109 30L210 40M109 50L210 40M109 50L210 60M210 20L312 10M210 20L312 30M210 40L312 30M210 40L312 50M210 60L312 50M210 60L312 70",
  placement:
    "M1 4H120V14H1M120 9H240M1 24H120V34H1M120 29H240M1 44H120V54H1M120 49H240M1 64H120V74H1M120 69H240",
};

/**
 * Os jogadores, nos formatos em que eles são pontos ligados por traços: cada
 * ponto é um traço de comprimento zero com a ponta redonda, que continua
 * redondo quando o quadro estica.
 */
const DOTS: Partial<Record<BracketDiagramFormat, string>> = {
  round_robin: "M40 74h0M100 74h0M160 74h0M220 74h0M280 74h0",
  swiss:
    "M8 40h0M109 30h0M109 50h0M210 20h0M210 40h0M210 60h0M312 10h0M312 30h0M312 50h0M312 70h0",
};

/** Só retas na horizontal e na vertical: sem suavizar, ficam nítidas. */
const STRAIGHT: BracketDiagramFormat[] = ["single", "double", "placement"];

/**
 * Um desenho mínimo do formato, entre o nome e a frase do tile do
 * Chaveamento, para achá-lo de relance (o nome do tile já diz qual é: fica
 * fora do leitor de tela). Traço fino e uniforme, sem preenchimento, na cor
 * que o tile dá (`currentColor`); na largura do tile e na mesma altura em
 * todos, para ficar alinhado na fileira.
 */
export function BracketDiagram({ format }: { format: BracketDiagramFormat }) {
  const dots = DOTS[format];
  return (
    <svg
      className={cn(
        styles.diagram,
        STRAIGHT.includes(format) && styles.straight,
      )}
      viewBox="0 0 320 80"
      preserveAspectRatio="none"
      aria-hidden="true"
      focusable="false"
    >
      <path className={styles.lines} d={PATHS[format]} />
      {dots && <path className={styles.dots} d={dots} />}
    </svg>
  );
}
