import type { Meta, StoryObj } from "@storybook/nextjs-vite";

import { AppHeader } from "./app-header";
import { HeaderPlaceholder } from "./header-placeholder";
import { UserPanel } from "./user-panel";

const meta = {
  title: "Cabeçalho/HeaderPlaceholder",
  component: HeaderPlaceholder,
  parameters: {
    layout: "fullscreen",
    docs: { story: { inline: false, height: "200px" } },
  },
  decorators: [
    (Story) => (
      <AppHeader name="Telefrag" skipLabel="Pular para o conteúdo">
        <Story />
      </AppHeader>
    ),
  ],
} satisfies Meta<typeof HeaderPlaceholder>;

export default meta;
type Story = StoryObj<typeof meta>;

/** Invisível: só segura os 48px de uma ação da barra enquanto ela carrega. */
export const Default: Story = {};

/** A pessoa já chegou e os avisos ainda carregam: a barra não pula. */
export const NextToAction: Story = {
  render: () => (
    <>
      <HeaderPlaceholder />
      <UserPanel
        label="Entrar ou criar conta"
        items={[
          { label: "Entrar", href: "/login" },
          { label: "Criar conta", href: "/signup" },
        ]}
      />
    </>
  ),
};
