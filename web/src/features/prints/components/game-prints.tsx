import { cn } from "@/shared/lib/cn";
import { loadMatchPrints } from "../server";
import { AttachPrint } from "./attach-print";
import styles from "./game-prints.module.scss";
import { PrintThumbnail } from "./print-thumbnail";

type Props = {
  /** A partida (o `MatchTarget` da página). */
  target: { tenant: string; slug: string; matchKey: string };
  number: number;
  /** Quem vê vai enviar este mapa: com print obrigatório, diz isso no anexo. */
  reporting?: boolean;
  /** Para onde vai o foco quando o controle usado some (a linha do mapa). */
  focusId: string;
  className?: string;
};

/**
 * Os prints de um mapa: miniaturas que abrem em tamanho real, com quem
 * anexou e "Remover" para quem pode, e "Anexar print" enquanto cabe mais um.
 * Some quando esta API não tem prints, ou o mapa não tem print nem como
 * anexar. Os mapas da página leem do mesmo pedido (`loadMatchPrints`).
 */
export async function GamePrints({
  target,
  number,
  reporting = false,
  focusId,
  className,
}: Props) {
  const all = await loadMatchPrints(
    target.tenant,
    target.slug,
    target.matchKey,
  );
  const game = all?.enabled
    ? all.games.find((g) => g.number === number)
    : undefined;
  if (!all || !game || (game.prints.length === 0 && !game.can_attach)) {
    return null;
  }
  const printTarget = { ...target, number };

  return (
    <div className={cn(styles.prints, className)}>
      {game.prints.length > 0 && (
        <ul className={styles.list}>
          {game.prints.map((print, i) => (
            <li key={print.id}>
              <PrintThumbnail
                target={printTarget}
                print={print}
                index={i + 1}
                focusId={focusId}
              />
            </li>
          ))}
        </ul>
      )}
      {game.can_attach && (
        <AttachPrint
          target={printTarget}
          maxBytes={all.max_bytes}
          required={reporting && all.required}
          focusId={focusId}
        />
      )}
    </div>
  );
}
