import type { Meta, StoryObj } from "@storybook/nextjs-vite";

import { Stack } from "./carbon/stack";
import { type Status, StatusTag } from "./status-tag";

const labels: Record<Status, string> = {
  cancelled: "Desistiu",
  contested: "Contestada",
  action: "Sua vez no veto",
  neutral: "Você",
};

const statuses = Object.keys(labels) as Status[];

const meta = {
  title: "Estado e conteúdo/StatusTag",
  component: StatusTag,
  args: { status: "cancelled", children: "Desistiu" },
  argTypes: {
    status: { control: "select", options: statuses },
    size: { control: "inline-radio", options: ["sm", "md"] },
    children: { control: "text" },
  },
} satisfies Meta<typeof StatusTag>;

export default meta;
type Story = StoryObj<typeof meta>;

/** Saiu, desistiu, ficou fora da chave. */
export const Cancelled: Story = {
  args: { status: "cancelled", children: labels.cancelled },
};

/** Contestado. */
export const Contested: Story = {
  args: { status: "contested", children: labels.contested },
};

/** A ação que falta a quem vê, no cartão da série. */
export const Action: Story = {
  args: { status: "action", children: labels.action },
};

/** Marcador sem gravidade: rascunho, "Você", reserva. */
export const Neutral: Story = {
  args: { status: "neutral", children: labels.neutral },
};

/** Todos os estados nos dois tamanhos: a cor fixa de cada significado. */
export const Statuses: Story = {
  render: (args) => (
    <Stack gap={5}>
      {(["md", "sm"] as const).map((size) => (
        <Stack key={size} orientation="horizontal" gap={3}>
          {statuses.map((status) => (
            <StatusTag key={status} {...args} status={status} size={size}>
              {labels[status]}
            </StatusTag>
          ))}
        </Stack>
      ))}
    </Stack>
  ),
};

export const Sizes: Story = {
  render: (args) => (
    <Stack orientation="horizontal" gap={3}>
      {(["md", "sm"] as const).map((size) => (
        <StatusTag key={size} {...args} size={size}>
          {size}
        </StatusTag>
      ))}
    </Stack>
  ),
};
