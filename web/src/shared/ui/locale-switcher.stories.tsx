import type { Meta, StoryObj } from "@storybook/nextjs-vite";

import { LocaleSwitcher } from "./locale-switcher";

const meta = {
  title: "Página/LocaleSwitcher",
  component: LocaleSwitcher,
  parameters: {
    nextjs: { navigation: { pathname: "/pt/arena-brasil/copa-de-duelo" } },
  },
} satisfies Meta<typeof LocaleSwitcher>;

export default meta;
type Story = StoryObj<typeof meta>;

/** Cada idioma no próprio nome; o atual (o da barra de ferramentas) é texto, os outros são links. */
export const Default: Story = {};

export const Spanish: Story = {
  globals: { locale: "es" },
  parameters: {
    nextjs: { navigation: { pathname: "/es/arena-brasil/copa-de-duelo" } },
  },
};

export const English: Story = {
  globals: { locale: "en" },
  parameters: {
    nextjs: { navigation: { pathname: "/en/arena-brasil/copa-de-duelo" } },
  },
};
