import type { Meta, StoryObj } from "@storybook/nextjs-vite";

import { Stack } from "./carbon/stack";
import { ShapeIndicator } from "./shape-indicator";

const labels = {
  failed: "Falhou",
  critical: "Crítico",
  high: "Alto",
  medium: "Médio",
  low: "Baixo",
  cautious: "Cuidado",
  undefined: "Indefinido",
  stable: "Estável",
  informative: "Informativo",
  incomplete: "Incompleto",
  draft: "Rascunho",
} as const;

const kinds = Object.keys(labels) as (keyof typeof labels)[];

const meta = {
  title: "Estado e conteúdo/ShapeIndicator",
  component: ShapeIndicator,
  args: { kind: "high", label: labels.high },
  argTypes: {
    kind: { control: "select", options: kinds },
    textSize: { control: "inline-radio", options: [12, 14] },
    label: { control: "text" },
  },
} satisfies Meta<typeof ShapeIndicator>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};

/** Todos os níveis: a forma muda junto da cor. */
export const Kinds: Story = {
  render: (args) => (
    <Stack gap={3}>
      {kinds.map((kind) => (
        <ShapeIndicator key={kind} {...args} kind={kind} label={labels[kind]} />
      ))}
    </Stack>
  ),
};

export const TextSize14: Story = {
  args: { textSize: 14 },
  render: Kinds.render,
};

/** Só a forma: o texto aparece na dica e vai para o leitor de tela. */
export const Compact: Story = {
  args: { compact: true },
  render: (args) => (
    <Stack orientation="horizontal" gap={3}>
      {kinds.map((kind) => (
        <ShapeIndicator key={kind} {...args} kind={kind} label={labels[kind]} />
      ))}
    </Stack>
  ),
};
