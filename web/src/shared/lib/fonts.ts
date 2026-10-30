import { IBM_Plex_Mono, IBM_Plex_Sans } from "next/font/google";

// IBM Plex, a fonte do Carbon, servida pelo next/font (sem CDN). As
// variáveis são lidas pelas famílias do Carbon em globals.scss. Ficam aqui
// porque há mais de um `<html>`: o do layout e os das páginas de erro que o
// substituem (`global-error` e o `not-found` da raiz).
export const plexSans = IBM_Plex_Sans({
  variable: "--font-plex-sans",
  subsets: ["latin"],
});

export const plexMono = IBM_Plex_Mono({
  variable: "--font-plex-mono",
  subsets: ["latin"],
  weight: ["400", "600"],
});
