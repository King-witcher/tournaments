import type { Meta, StoryObj } from "@storybook/nextjs-vite";
import { userEvent, within } from "storybook/test";

import { AppHeader } from "./app-header";
import { LocaleSwitcher } from "./locale-switcher";

const meta = {
  title: "Cabeçalho/LocaleSwitcher",
  component: LocaleSwitcher,
  parameters: {
    layout: "fullscreen",
    // O painel é fixo, abaixo da barra.
    docs: { story: { inline: false, height: "300px" } },
    nextjs: { navigation: { pathname: "/pt/arena-brasil/copa-de-duelo" } },
  },
  decorators: [
    (Story) => (
      <AppHeader name="Telefrag" skipLabel="Pular para o conteúdo">
        <Story />
      </AppHeader>
    ),
  ],
} satisfies Meta<typeof LocaleSwitcher>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Closed: Story = {};

/** Cada idioma no próprio nome; o atual (o da barra de ferramentas) fica marcado. */
export const Open: Story = {
  play: async ({ canvasElement }) => {
    await userEvent.click(
      await within(canvasElement).findByRole("button", { expanded: false }),
    );
  },
};

export const OpenSpanish: Story = {
  ...Open,
  globals: { locale: "es" },
};
