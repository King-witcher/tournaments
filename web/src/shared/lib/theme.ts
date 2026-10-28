export const THEME_KEY = "theme";
/** Na ordem do seletor: claro, o do navegador e escuro. */
export const THEME_CHOICES = ["light", "system", "dark"] as const;
export type ThemeChoice = (typeof THEME_CHOICES)[number];

/**
 * Roda no `<head>`, antes da primeira pintura: aplica a escolha guardada em
 * `data-theme` no `<html>` (sem ela, vale o tema do navegador, em
 * `globals.scss`). Evita o clarão do tema errado ao carregar.
 */
export const themeInitScript = `try{var t=localStorage.getItem("${THEME_KEY}");if(t==="light"||t==="dark")document.documentElement.dataset.theme=t}catch(e){}`;
