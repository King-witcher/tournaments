import type { Meta, StoryObj } from "@storybook/nextjs-vite";

import { ThemeSwitcher } from "./theme-switcher";

const meta = {
  title: "Cabeçalho/ThemeSwitcher",
  component: ThemeSwitcher,
} satisfies Meta<typeof ThemeSwitcher>;

export default meta;
type Story = StoryObj<typeof meta>;

/**
 * Claro, o do navegador e escuro. Escolher grava `data-theme` no `<html>` e no
 * `localStorage`, e disputa com o tema da barra de ferramentas.
 */
export const Default: Story = {};

export const Spanish: Story = { globals: { locale: "es" } };
