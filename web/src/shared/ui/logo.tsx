import { cn } from "@/shared/lib/cn";
import {
  ICON_PATH,
  ICON_WIDTH,
  LOGO_HEIGHT,
  LOGO_WIDTH,
  WORDMARK_PATH,
} from "@/shared/lib/logo";
import styles from "./logo.module.scss";

/**
 * O logo do Telefrag na cor do texto, na altura do cabeçalho. É decorativo:
 * quem o envolve dá o nome acessível (`aria-label` no link). Ícone e nome
 * são dois desenhos com a mesma altura de quadro, para ficarem alinhados
 * como no banner; no celular, só o ícone, e a barra de ações cabe.
 */
export function Logo() {
  return (
    <span className={styles.logo}>
      <svg
        viewBox={`0 0 ${ICON_WIDTH} ${LOGO_HEIGHT}`}
        className={styles.part}
        aria-hidden="true"
      >
        <path d={ICON_PATH} />
      </svg>
      <svg
        viewBox={`${ICON_WIDTH} 0 ${LOGO_WIDTH - ICON_WIDTH} ${LOGO_HEIGHT}`}
        className={cn(styles.part, styles.wordmark)}
        aria-hidden="true"
      >
        <path d={WORDMARK_PATH} />
      </svg>
    </span>
  );
}
