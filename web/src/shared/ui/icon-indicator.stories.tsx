import type { Meta, StoryObj } from "@storybook/nextjs-vite";

import { Stack } from "./carbon/stack";
import { IconIndicator } from "./icon-indicator";

const labels = {
  failed: "Falhou",
  "caution-major": "Contestada",
  "caution-minor": "Atrasada",
  undefined: "Indefinida",
  succeeded: "Confirmada",
  normal: "Normal",
  "in-progress": "Em andamento",
  incomplete: "Incompleta",
  "not-started": "Não começou",
  pending: "Aguardando",
  unknown: "Desconhecida",
  informative: "Informação",
} as const;

const kinds = Object.keys(labels) as (keyof typeof labels)[];

const meta = {
  title: "Estado e conteúdo/IconIndicator",
  component: IconIndicator,
  args: { kind: "in-progress", label: labels["in-progress"] },
  argTypes: {
    kind: { control: "select", options: kinds },
    size: { control: "inline-radio", options: [16, 20] },
    label: { control: "text" },
  },
} satisfies Meta<typeof IconIndicator>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};

/** Todos os estados, com os textos de uma série. */
export const Kinds: Story = {
  render: (args) => (
    <Stack gap={3}>
      {kinds.map((kind) => (
        <IconIndicator key={kind} {...args} kind={kind} label={labels[kind]} />
      ))}
    </Stack>
  ),
};

export const Size20: Story = {
  args: { size: 20 },
  render: Kinds.render,
};

/** Só o ícone: o texto aparece na dica e vai para o leitor de tela. */
export const Compact: Story = {
  args: { compact: true },
  render: (args) => (
    <Stack orientation="horizontal" gap={3}>
      {kinds.map((kind) => (
        <IconIndicator key={kind} {...args} kind={kind} label={labels[kind]} />
      ))}
    </Stack>
  ),
};
