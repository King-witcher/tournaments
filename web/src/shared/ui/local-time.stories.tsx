import { Stack } from "@carbon/react";
import type { Meta, StoryObj } from "@storybook/nextjs-vite";

import { LocalTime } from "./local-time";

/** O instante, em ISO, hoje às `hour`:`minute` locais. */
function today(hour: number, minute = 0) {
  const date = new Date();
  date.setHours(hour, minute, 0, 0);
  return date.toISOString();
}

const meta = {
  title: "Data e hora/LocalTime",
  component: LocalTime,
  args: { value: today(21) },
  argTypes: { value: { control: "text" } },
} satisfies Meta<typeof LocalTime>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Evening: Story = {};

export const Morning: Story = { args: { value: today(9, 30) } };

export const Midnight: Story = { args: { value: today(0) } };

/** Só a hora: serve para prazos do mesmo dia. */
export const InSentence: Story = {
  render: (args) => (
    <p>
      Faça o check-in até as <LocalTime {...args} />.
    </p>
  ),
};

export const Overview: Story = {
  render: () => (
    <Stack gap={3}>
      <LocalTime value={today(9, 30)} />
      <LocalTime value={today(21)} />
      <LocalTime value={today(0)} />
    </Stack>
  ),
};
