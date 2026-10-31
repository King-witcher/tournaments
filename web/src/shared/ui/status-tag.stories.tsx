import { Stack } from "@carbon/react";
import type { Meta, StoryObj } from "@storybook/nextjs-vite";

import { type Status, StatusTag } from "./status-tag";

const labels: Record<Status, string> = {
  draft: "Rascunho",
  registration: "Inscrições abertas",
  check_in: "Check-in",
  in_progress: "Em andamento",
  finished: "Encerrado",
  cancelled: "Cancelado",
  waiting: "Aguardando",
  contested: "Contestado",
  confirmed: "Confirmado",
  neutral: "Organizador",
};

const statuses = Object.keys(labels) as Status[];

const meta = {
  title: "Estado e conteúdo/StatusTag",
  component: StatusTag,
  args: { status: "in_progress", children: "Em andamento" },
  argTypes: {
    status: { control: "select", options: statuses },
    size: { control: "inline-radio", options: ["sm", "md"] },
    children: { control: "text" },
  },
} satisfies Meta<typeof StatusTag>;

export default meta;
type Story = StoryObj<typeof meta>;

/** Rascunho. */
export const Draft: Story = {
  args: { status: "draft", children: labels.draft },
};

/** Inscrições abertas, inscrito. */
export const Registration: Story = {
  args: { status: "registration", children: labels.registration },
};

/** Check-in aberto ou pendente. */
export const CheckIn: Story = {
  args: { status: "check_in", children: labels.check_in },
};

/** Em andamento, jogando. */
export const InProgress: Story = {
  args: { status: "in_progress", children: labels.in_progress },
};

/** Encerrado, eliminado. */
export const Finished: Story = {
  args: { status: "finished", children: labels.finished },
};

/** Cancelado, desistiu, saiu, perdeu a vaga. */
export const Cancelled: Story = {
  args: { status: "cancelled", children: labels.cancelled },
};

/** Aguardando o outro lado ou o prazo. */
export const Waiting: Story = {
  args: { status: "waiting", children: labels.waiting },
};

export const Contested: Story = {
  args: { status: "contested", children: labels.contested },
};

/** Confirmado, check-in feito, venceu. */
export const Confirmed: Story = {
  args: { status: "confirmed", children: labels.confirmed },
};

/** Sem estado especial (papel, contagem). */
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
