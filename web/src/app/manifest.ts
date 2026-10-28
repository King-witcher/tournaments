import type { MetadataRoute } from "next";

import metadata from "../../messages/pt/Metadata.json";
import nav from "../../messages/pt/Nav.json";

/**
 * O manifest do site (`/manifest.webmanifest`): instalável na tela inicial,
 * que é o que liga o Web Push no iPhone. Um só para os dois idiomas, com o
 * nome do cabeçalho no idioma padrão; `/` leva cada um ao seu idioma.
 */
export default function manifest(): MetadataRoute.Manifest {
  return {
    name: nav.brand,
    short_name: nav.brand,
    description: metadata.description,
    start_url: "/",
    scope: "/",
    display: "standalone",
    // As cores do tema claro do Carbon: o fundo (white) e o do ícone (blue 60).
    background_color: "#ffffff",
    theme_color: "#ffffff",
    icons: [
      { src: "/icon", sizes: "512x512", type: "image/png", purpose: "any" },
      { src: "/apple-icon", sizes: "180x180", type: "image/png" },
    ],
  };
}
