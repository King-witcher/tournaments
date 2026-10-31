import { Stack } from "@carbon/react";
import type { Meta, StoryObj } from "@storybook/nextjs-vite";

import { ProgressSteps } from "./progress-steps";

const steps = [
  { key: "draft", label: "Rascunho" },
  { key: "registration", label: "Inscrições" },
  { key: "check_in", label: "Check-in" },
  { key: "in_progress", label: "Em andamento" },
  { key: "finished", label: "Encerrado" },
];

const meta = {
  title: "Estado e conteúdo/ProgressSteps",
  component: ProgressSteps,
  args: { steps, current: 2, label: "Ciclo do torneio" },
  argTypes: {
    current: {
      control: { type: "number", min: 0, max: steps.length },
    },
    steps: { control: false },
  },
} satisfies Meta<typeof ProgressSteps>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};

/** Nenhum passo feito. */
export const First: Story = { args: { current: 0 } };

export const Last: Story = { args: { current: steps.length - 1 } };

/** Com `current` além do último passo, todos aparecem como feitos. */
export const AllDone: Story = { args: { current: steps.length } };

/** A linha embaixo do nome: uma data, "agora". */
export const WithSecondaryLabels: Story = {
  args: {
    steps: [
      { key: "draft", label: "Rascunho", secondaryLabel: "05/11" },
      { key: "registration", label: "Inscrições", secondaryLabel: "até 11/11" },
      { key: "check_in", label: "Check-in", secondaryLabel: "12/11, 20h30" },
      {
        key: "in_progress",
        label: "Em andamento",
        secondaryLabel: "12/11, 21h",
      },
      { key: "finished", label: "Encerrado" },
    ],
    current: 2,
  },
};

/** Cada passo atual, para ver feito, atual e pendente em sequência. */
export const EachStep: Story = {
  render: (args) => (
    <Stack gap={7}>
      {steps.map((step, index) => (
        <ProgressSteps key={step.key} {...args} current={index} />
      ))}
    </Stack>
  ),
};

/** Até `md` (celular), a lista fica vertical. */
export const Phone: Story = {
  args: WithSecondaryLabels.args,
  globals: { viewport: { value: "phone", isRotated: false } },
};
