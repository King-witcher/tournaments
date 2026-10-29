import { cn } from "@/shared/lib/cn";
import { loadMatchScreenshots } from "../server";
import { AttachScreenshot } from "./attach-screenshot";
import styles from "./game-screenshots.module.scss";
import { ScreenshotThumbnail } from "./screenshot-thumbnail";

type Props = {
  /** A partida (o `MatchTarget` da página). */
  target: { tenant: string; slug: string; matchKey: string };
  number: number;
  /** Quem vê vai enviar este mapa: com screenshot obrigatório, diz isso no anexo. */
  reporting?: boolean;
  /** Para onde vai o foco quando o controle usado some (a linha do mapa). */
  focusId: string;
  className?: string;
};

/**
 * Os screenshots de um mapa: miniaturas que abrem em tamanho real, com quem
 * anexou e "Remover" para quem pode, e "Anexar screenshot" enquanto cabe mais
 * um. Some quando esta API não tem screenshots, ou o mapa não tem screenshot
 * nem como anexar. Os mapas da página leem do mesmo pedido
 * (`loadMatchScreenshots`).
 */
export async function GameScreenshots({
  target,
  number,
  reporting = false,
  focusId,
  className,
}: Props) {
  const all = await loadMatchScreenshots(
    target.tenant,
    target.slug,
    target.matchKey,
  );
  const game = all?.enabled
    ? all.games.find((g) => g.number === number)
    : undefined;
  if (!all || !game || (game.screenshots.length === 0 && !game.can_attach)) {
    return null;
  }
  const screenshotTarget = { ...target, number };

  return (
    <div className={cn(styles.screenshots, className)}>
      {game.screenshots.length > 0 && (
        <ul className={styles.list}>
          {game.screenshots.map((screenshot, i) => (
            <li key={screenshot.id}>
              <ScreenshotThumbnail
                target={screenshotTarget}
                screenshot={screenshot}
                index={i + 1}
                focusId={focusId}
              />
            </li>
          ))}
        </ul>
      )}
      {game.can_attach && (
        <AttachScreenshot
          target={screenshotTarget}
          maxBytes={all.max_bytes}
          required={reporting && all.required}
          focusId={focusId}
        />
      )}
    </div>
  );
}
