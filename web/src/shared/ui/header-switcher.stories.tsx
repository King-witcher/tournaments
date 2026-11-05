import { UserMultiple } from "@carbon/icons-react";
import type { Meta, StoryObj } from "@storybook/nextjs-vite";
import { userEvent, within } from "storybook/test";

import { AppHeader } from "./app-header";
import { SwitcherDivider, SwitcherItem } from "./carbon/ui-shell";
import { HeaderPanelAction } from "./header-panel-action";
import { HeaderSwitcher } from "./header-switcher";

const communities = [
  "Arena Brasil",
  "Liga Sul-Americana de Duelo e Times, temporada de primavera de 2026",
  "Quake Champions Chile",
];

const meta = {
  title: "Cabeçalho/HeaderSwitcher",
  component: HeaderSwitcher,
  parameters: {
    layout: "fullscreen",
    // O painel é fixo, abaixo da barra.
    docs: { story: { inline: false, height: "400px" } },
  },
  decorators: [
    (Story) => (
      <AppHeader name="Telefrag" skipLabel="Pular para o conteúdo">
        <HeaderPanelAction
          label="Suas comunidades"
          icon={<UserMultiple size={20} />}
          navigation
        >
          <Story />
        </HeaderPanelAction>
      </AppHeader>
    ),
  ],
  args: {
    label: "Suas comunidades",
    children: (
      <>
        {communities.map((name, index) => (
          <SwitcherItem
            key={name}
            href="/arena-brasil"
            aria-label={name}
            aria-current={index === 0 ? "true" : undefined}
          >
            {name}
          </SwitcherItem>
        ))}
        <SwitcherDivider />
        <SwitcherItem href="/new" aria-label="Criar comunidade">
          Criar comunidade
        </SwitcherItem>
      </>
    ),
  },
  argTypes: { children: { control: false } },
  play: async ({ canvasElement }) => {
    await userEvent.click(
      await within(canvasElement).findByRole("button", { expanded: false }),
    );
  },
} satisfies Meta<typeof HeaderSwitcher>;

export default meta;
type Story = StoryObj<typeof meta>;

/**
 * O nome longo quebra a linha. A comunidade da página atual (a primeira) não
 * tem marca visível, só `aria-current`.
 */
export const Default: Story = {};

/** No celular, cada item tem 48px. */
export const Phone: Story = {
  globals: { viewport: { value: "phone", isRotated: false } },
};
