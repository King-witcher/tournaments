import type { Meta, StoryObj } from "@storybook/nextjs-vite";

import { DefinitionTooltip } from "./definition-tooltip";

const alignments = [
  "top",
  "top-start",
  "top-end",
  "bottom",
  "bottom-start",
  "bottom-end",
  "left",
  "left-start",
  "left-end",
  "right",
  "right-start",
  "right-end",
] as const;

const meta = {
  title: "Estado e conteúdo/DefinitionTooltip",
  component: DefinitionTooltip,
  args: {
    definition:
      "Vitória dada ao lado que compareceu quando o outro não aparece no prazo.",
    children: "W.O.",
  },
  argTypes: {
    align: { control: "select", options: alignments },
    definition: { control: "text" },
    children: { control: "text" },
  },
  // O termo vai no meio de uma frase.
  render: (args) => (
    <p>
      A série termina por <DefinitionTooltip {...args} /> se um dos lados não
      fizer o check-in.
    </p>
  ),
} satisfies Meta<typeof DefinitionTooltip>;

export default meta;
type Story = StoryObj<typeof meta>;

/** Abre ao clicar ou focar o termo. */
export const Default: Story = {};

/** Também ao passar o mouse. */
export const OpenOnHover: Story = { args: { openOnHover: true } };

/** Aberta, como fica depois do clique. */
export const Open: Story = {
  args: { defaultOpen: true },
  parameters: { docs: { story: { inline: false, height: "200px" } } },
};

export const WithLargeText: Story = {
  args: {
    defaultOpen: true,
    children: "queda do K",
    definition:
      "O fator K diminui conforme o jogador acumula partidas: o rating de quem joga muito muda menos a cada resultado. Quem está começando sobe ou desce mais rápido.",
  },
  parameters: Open.parameters,
};

export const Top: Story = {
  args: { defaultOpen: true, align: "top" },
  parameters: { docs: { story: { inline: false, height: "200px" } } },
};

/** Sem espaço no lado pedido, a caixa troca de lado sozinha. */
export const AutoAlign: Story = {
  args: { defaultOpen: true, align: "top", autoAlign: true },
  parameters: Open.parameters,
};
