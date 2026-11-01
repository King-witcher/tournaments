import { Stack } from "@carbon/react";
import type { Meta, StoryObj } from "@storybook/nextjs-vite";
import { fn } from "storybook/test";

import { CopyButton } from "./copy-button";

const meta = {
  title: "Ação/CopyButton",
  component: CopyButton,
  args: { text: "arena-sul-copa-2026", onClick: fn() },
  argTypes: {
    align: {
      control: "select",
      options: ["top", "bottom", "left", "right"],
    },
    text: { control: "text" },
    onClick: { control: false },
  },
} satisfies Meta<typeof CopyButton>;

export default meta;
type Story = StoryObj<typeof meta>;

/** Clique: copia o `text` e mostra "Copiado" na dica. */
export const Default: Story = {};

/** Ao lado do que copia, que já está à vista. */
export const NextToText: Story = {
  render: (args) => (
    <Stack orientation="horizontal" gap={3}>
      <span>{args.text}</span>
      <CopyButton {...args} />
    </Stack>
  ),
};

export const AlignTop: Story = { args: { align: "top" } };

export const AlignRight: Story = { args: { align: "right" } };

/** A dica "Copiado" fica 5 segundos (o padrão é 2). */
export const LongFeedback: Story = { args: { feedbackTimeout: 5000 } };

export const Disabled: Story = { args: { disabled: true } };
