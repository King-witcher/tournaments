import { Tile } from "@carbon/react";
import type { Meta, StoryObj } from "@storybook/nextjs-vite";
import { fn } from "storybook/test";

import { AppContent } from "./app-content";
import { Button } from "./button";
import { Page } from "./page";
import { Section } from "./section";

const meta = {
  title: "Página/Section",
  component: Section,
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
    title: "Inscritos",
    children: <Tile>18 jogadores inscritos de 32 vagas.</Tile>,
  },
  argTypes: {
    title: { control: "text" },
    description: { control: "text" },
    level: { control: "inline-radio", options: [2, 3] },
    action: { control: false },
    children: { control: false },
  },
} satisfies Meta<typeof Section>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};

export const WithDescription: Story = {
  args: { description: "As vagas fecham no check-in." },
};

export const WithAction: Story = {
  args: {
    action: (
      <Button kind="tertiary" onClick={fn()}>
        Gerar link
      </Button>
    ),
  },
};

export const WithDescriptionAndAction: Story = {
  args: { ...WithDescription.args, ...WithAction.args },
};

/** Título `h3`, para a seção dentro de outra. */
export const Level3: Story = { args: { level: 3, title: "Excluir torneio" } };

export const Nested: Story = {
  args: {
    title: "Configurações",
    children: (
      <Section level={3} title="Zona de perigo">
        <Tile>Excluir o torneio apaga as inscrições e as séries.</Tile>
      </Section>
    ),
  },
};

/** No celular a ação desce para baixo do título. */
export const Phone: Story = {
  ...WithDescriptionAndAction,
  globals: { viewport: { value: "phone", isRotated: false } },
  args: {
    ...WithDescriptionAndAction.args,
    title: "Links de inscrição da comunidade Arena Brasil",
  },
};
