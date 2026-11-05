import { Notification } from "@carbon/icons-react";
import type { Meta, StoryObj } from "@storybook/nextjs-vite";
import { expect, userEvent, within } from "storybook/test";

import { AppHeader } from "./app-header";
import { SwitcherItem } from "./carbon/ui-shell";
import { HeaderPanelAction } from "./header-panel-action";
import { HeaderSwitcher } from "./header-switcher";

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
      <HeaderSwitcher label="Avisos">
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
      </HeaderSwitcher>
    ),
  },
  argTypes: {
    label: { control: "text" },
    text: { control: "text" },
    badgeCount: { control: "number" },
    navigation: { control: "boolean" },
    icon: { control: false },
    children: { control: false },
  },
} satisfies Meta<typeof HeaderPanelAction>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Closed: Story = {};

/**
 * Aberto pelo teclado, o foco vai ao primeiro item; ↑ e ↓ andam pelos itens,
 * e Esc fecha e devolve o foco ao botão.
 */
export const Open: Story = {
  play: async ({ canvasElement }) => {
    const canvas = within(canvasElement);
    (await canvas.findByRole("button", { expanded: false })).focus();
    await userEvent.keyboard("{Enter}");
    await expect(
      canvas.getByRole("link", { name: "Copa começa hoje" }),
    ).toHaveFocus();
  },
};

/** Aberto pelo mouse, o foco fica no botão: nenhum item com o contorno. */
export const OpenWithMouse: Story = {
  play: async ({ canvasElement }) => {
    const canvas = within(canvasElement);
    const button = await canvas.findByRole("button", { expanded: false });
    await userEvent.click(button);
    await expect(button).toHaveAttribute("aria-expanded", "true");
  },
};

/** O número no canto do ícone (o `BadgeIndicator` do Carbon). */
export const WithBadge: Story = {
  args: { badgeCount: 3 },
};

/** Acima de 999, "999+". */
export const WithLargeBadge: Story = {
  args: { badgeCount: 1500 },
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
