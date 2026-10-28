import styles from "./header-placeholder.module.scss";

/** Fallback do `<Suspense>` de uma ação do `AppHeader` (idioma, a pessoa). */
export function HeaderPlaceholder() {
  return <span aria-hidden className={styles.placeholder} />;
}
