import { Notification } from "@carbon/icons-react";
import { Switcher, SwitcherItem } from "@carbon/react";
import type { Meta, StoryObj } from "@storybook/nextjs-vite";
import { expect, userEvent, within } from "storybook/test";

import { AppHeader } from "./app-header";
import { HeaderPanelAction } from "./header-panel-action";

const meta = {
  title: "Cabeçalho/HeaderPanelAction",
  component: HeaderPanelAction,
  parameters: {
    layout: "fullscreen",
    // O painel é fixo, abaixo da barra.
    docs: { story: { inline: false, height: "500px" } },
  },
  decorators: [
    (Story) => (
      <AppHeader name="Telefrag" skipLabel="Pular para o conteúdo">
        <Story />
      </AppHeader>
    ),
  ],
  args: {
    label: "Avisos",
    icon: <Notification size={20} />,
    children: (
      <Switcher aria-label="Avisos" expanded>
        <SwitcherItem
          href="/arena-brasil/copa-de-duelo"
          aria-label="Copa começa hoje"
        >
          Copa começa hoje
        </SwitcherItem>
        <SwitcherItem
          href="/arena-brasil/copa-de-duelo"
          aria-label="Série pronta"
        >
          Série pronta
        </SwitcherItem>
      </Switcher>
    ),
  },
  argTypes: {
    label: { control: "text" },
    text: { control: "text" },
    icon: { control: false },
    children: { control: false },
  },
} satisfies Meta<typeof HeaderPanelAction>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Closed: Story = {};

/** Abre no clique e leva o foco ao primeiro item; Esc fecha e devolve o foco ao botão. */
export const Open: Story = {
  play: async ({ canvasElement }) => {
    const canvas = within(canvasElement);
    await userEvent.click(
      await canvas.findByRole("button", { expanded: false }),
    );
    await expect(
      canvas.getByRole("link", { name: "Copa começa hoje" }),
    ).toHaveFocus();
  },
};

/** O texto ao lado do ícone termina em reticências quando não cabe. */
export const WithText: Story = {
  args: { text: "Lobo Solitário da Arena Brasil" },
  play: Open.play,
};

export const Phone: Story = {
  ...WithText,
  globals: { viewport: { value: "phone", isRotated: false } },
};
