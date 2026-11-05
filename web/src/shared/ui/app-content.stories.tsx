import type { Meta, StoryObj } from "@storybook/nextjs-vite";

import { AppContent } from "./app-content";
import { AppHeader } from "./app-header";
import { Tile } from "./carbon/tile";
import { Page } from "./page";
import { PageHeader } from "./page-header";
import { Section } from "./section";
import { UserPanel } from "./user-panel";

const meta = {
  title: "Cabeçalho/AppContent",
  component: AppContent,
  parameters: { layout: "fullscreen" },
  decorators: [
    (Story) => (
      <>
        <AppHeader name="Telefrag" skipLabel="Pular para o conteúdo">
          <UserPanel
            label="Entrar ou criar conta"
            items={[
              { label: "Entrar", href: "/login" },
              { label: "Criar conta", href: "/signup" },
            ]}
          />
        </AppHeader>
        <Story />
      </>
    ),
  ],
  args: {
    children: (
      <Page>
        <PageHeader title="Copa de Duelo" meta="Duelo · 18/32 inscritos" />
        <Section title="Inscritos">
          <Tile>18 jogadores inscritos de 32 vagas.</Tile>
        </Section>
      </Page>
    ),
  },
  argTypes: { children: { control: false } },
} satisfies Meta<typeof AppContent>;

export default meta;
type Story = StoryObj<typeof meta>;

/** O `<main id="main-content">`, abaixo do cabeçalho; as margens laterais são do `Page`. */
export const Default: Story = {};

export const WidePage: Story = {
  args: {
    children: (
      <Page width="wide">
        <PageHeader title="Chave" meta="Eliminatória simples · 32 jogadores" />
        <Section title="Quartas de final">
          <Tile>Série 1 · Lobo contra Raven.</Tile>
        </Section>
      </Page>
    ),
  },
};

/** O espaçamento em volta é menor no celular. */
export const Phone: Story = {
  globals: { viewport: { value: "phone", isRotated: false } },
};
