import type { Meta, StoryObj } from "@storybook/nextjs-vite";

import { AppContent } from "./app-content";
import { Page } from "./page";
import { PageError } from "./page-error";

const meta = {
  title: "Página/PageError",
  component: PageError,
  parameters: { layout: "fullscreen" },
  decorators: [
    (Story) => (
      <AppContent>
        <Page>
          <Story />
        </Page>
      </AppContent>
    ),
  ],
  args: {
    children: "Torneio não encontrado.",
    href: "/arena-brasil",
    label: "Voltar para a Arena Brasil",
  },
  argTypes: { children: { control: "text" } },
} satisfies Meta<typeof PageError>;

export default meta;
type Story = StoryObj<typeof meta>;

export const NotFound: Story = {};

export const NoAccess: Story = {
  args: {
    children: "Você não participa desta comunidade.",
    href: "/",
    label: "Ir para o início",
  },
};

export const Phone: Story = {
  globals: { viewport: { value: "phone", isRotated: false } },
};
