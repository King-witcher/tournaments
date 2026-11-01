import { Layer } from "@carbon/react";
import type { Meta, StoryObj } from "@storybook/nextjs-vite";
import { fn } from "storybook/test";

import { Button } from "./button";
import { ExpandableTile } from "./expandable-tile";

const meta = {
  title: "Formulário/ExpandableTile",
  component: ExpandableTile,
  args: {
    above: "Veto de mapas na md3.",
    below:
      "Cada lado bane um mapa e depois escolhe um. O mapa decisivo é o que sobra.",
  },
  argTypes: {
    above: { control: "text" },
    below: { control: "text" },
    onClick: { control: false },
    onKeyUp: { control: false },
  },
} satisfies Meta<typeof ExpandableTile>;

export default meta;
type Story = StoryObj<typeof meta>;

/** O tile todo abre e fecha. */
export const Default: Story = {};

/** Começa aberto. */
export const Expanded: Story = {
  args: { expanded: true },
};

/** Com o texto ao lado da seta. */
export const WithLabels: Story = {
  args: {
    tileCollapsedLabel: "Ver as regras",
    tileExpandedLabel: "Esconder as regras",
  },
};

/** Com um botão dentro, só a seta abre e fecha. */
export const WithInteractive: Story = {
  args: {
    below: (
      <>
        <p>O mapa decisivo é o que sobra.</p>
        <Button kind="ghost" size="sm" onClick={fn()}>
          Ver o veto
        </Button>
      </>
    ),
  },
};

/** Sobre um `Layer`, o tile troca o fundo para continuar visível. */
export const InLayer: Story = {
  render: (args) => (
    <Layer>
      <ExpandableTile {...args} />
    </Layer>
  ),
};
