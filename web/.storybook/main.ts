import path from "node:path";
import type { StorybookConfig } from "@storybook/nextjs-vite";
import { mergeConfig } from "vite";

const config: StorybookConfig = {
  stories: ["../src/**/*.stories.tsx"],
  addons: [
    "@storybook/addon-docs",
    "@storybook/addon-a11y",
    "@storybook/addon-themes",
  ],
  framework: "@storybook/nextjs-vite",
  staticDirs: ["../public"],
  core: { disableTelemetry: true },
  // O Vite não passa pelo sass-loader do next.config.ts: o Sass do Carbon
  // precisa do node_modules como load path aqui também (e src, para as
  // nossas parciais), e os avisos de depreciação dele não são nossos.
  viteFinal: (vite) =>
    mergeConfig(vite, {
      css: {
        preprocessorOptions: {
          scss: {
            loadPaths: [path.resolve("node_modules"), path.resolve("src")],
            quietDeps: true,
          },
        },
      },
    }),
};

export default config;
