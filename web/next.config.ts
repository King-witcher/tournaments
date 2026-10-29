import path from "node:path";
import type { NextConfig } from "next";
import createNextIntlPlugin from "next-intl/plugin";

// O Sass do Carbon (node_modules/@carbon/*/scss) usa @use relativos com
// subpasta ('./theme/theme', 'scss/generated/...') que o importador do
// sass-loader embutido no Turbopack não acha. Aqui o Sass resolve tudo
// sozinho, como na linha de comando: sem o importador do webpack, com
// node_modules como load path (é assim que `@use "@carbon/react/..."` acha
// o pacote).
const sassLoader = {
  loader: "sass-loader",
  options: {
    api: "modern",
    webpackImporter: false,
    sassOptions: {
      loadPaths: [path.resolve("node_modules")],
      // Os avisos de depreciação do Sass dentro do Carbon não são nossos.
      quietDeps: true,
    },
  },
};

const nextConfig: NextConfig = {
  cacheComponents: true,
  partialPrefetching: true,
  // Nenhuma página abre dentro de um iframe (clickjacking: outro site
  // esconderia a página e faria a pessoa clicar nela sem ver). O
  // `frame-ancestors` é o atual; o `X-Frame-Options`, o dos navegadores
  // antigos. O CSP só tem essa diretiva, então não mexe em script nenhum.
  async headers() {
    return [
      {
        source: "/(.*)",
        headers: [
          { key: "Content-Security-Policy", value: "frame-ancestors 'none'" },
          { key: "X-Frame-Options", value: "DENY" },
          { key: "X-Content-Type-Options", value: "nosniff" },
          { key: "Referrer-Policy", value: "strict-origin-when-cross-origin" },
        ],
      },
    ];
  },
  images: {
    // Avatares do Discord.
    remotePatterns: [
      {
        protocol: "https",
        hostname: "cdn.discordapp.com",
        pathname: "/avatars/**",
      },
    ],
  },
  experimental: {
    // Os ícones do Carbon vêm de um índice com milhares de módulos: importa
    // só os usados. (@carbon/react não entra: o índice dele é que traz o
    // "use client" que deixa os componentes rodarem em Server Components.)
    optimizePackageImports: ["@carbon/icons-react"],
    // O .scss passa pelo sass-loader configurado abaixo, não pelo embutido.
    turbopackUseBuiltinSass: false,
  },
  turbopack: {
    rules: {
      // `x.module.scss` sai como `x.module.css` (CSS Module); o resto, como
      // CSS global. Todas as regras que casam rodam, por isso a condição.
      "*.module.scss": { loaders: [sassLoader], as: "*.module.css" },
      "*.scss": {
        condition: { not: { path: /\.module\.scss$/ } },
        loaders: [sassLoader],
        as: "*.css",
      },
    },
  },
};

const withNextIntl = createNextIntlPlugin();

export default withNextIntl(nextConfig);
