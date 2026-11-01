import type { Meta, StoryObj } from "@storybook/nextjs-vite";

import { ProgressStepsList } from "./progress-steps-list";

/**
 * O indicador do `ProgressSteps` (ele escolhe entre lado a lado e vertical
 * pela largura): aqui, as duas orientações à força.
 */
const meta = {
  title: "Estado e conteúdo/ProgressStepsList",
  component: ProgressStepsList,
  args: {
    steps: [
      { key: "ready", label: "Pronto" },
      { key: "game-1", label: "Partida 1", secondaryLabel: "25 × 14" },
      { key: "game-2", label: "Partida 2", secondaryLabel: "Agora" },
      { key: "game-3", label: "Partida 3" },
    ],
    current: 2,
    label: "Passos da série",
    vertical: false,
    selected: 1,
    onSelect: () => {},
  },
  argTypes: { steps: { control: false } },
} satisfies Meta<typeof ProgressStepsList>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Horizontal: Story = {};

export const Vertical: Story = { args: { vertical: true } };

/** Só leitura: sem `onSelect`, nenhum passo abre. */
export const ReadOnly: Story = {
  args: { onSelect: undefined, selected: undefined },
};
