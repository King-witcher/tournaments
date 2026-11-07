import styles from "./app-footer.module.scss";
import { Column, Grid } from "./carbon/grid";
import { LocaleSwitcher } from "./locale-switcher";

/**
 * O rodapé de todas as páginas, depois do `AppContent`, no fim da tela mesmo
 * em página curta (`globals.scss`). Só tem o idioma.
 */
export function AppFooter() {
  return (
    <footer className={styles.footer}>
      <Grid className={styles.grid}>
        <Column sm={4} md={8} lg={16}>
          <LocaleSwitcher />
        </Column>
      </Grid>
    </footer>
  );
}
