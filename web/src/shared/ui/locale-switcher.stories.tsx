import type { Meta, StoryObj } from "@storybook/nextjs-vite";

import { LocaleSwitcher } from "./locale-switcher";

const meta = {
  title: "Cabeçalho/LocaleSwitcher",
  component: LocaleSwitcher,
  parameters: {
    nextjs: { navigation: { pathname: "/pt/arena-brasil/copa-de-duelo" } },
  },
} satisfies Meta<typeof LocaleSwitcher>;

export default meta;
type Story = StoryObj<typeof meta>;

/** Cada idioma no próprio nome; o atual (o da barra de ferramentas) fica marcado. */
export const Default: Story = {};

export const Spanish: Story = { globals: { locale: "es" } };
export const English: Story = { globals: { locale: "en" } };
