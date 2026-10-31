import { Stack } from "@carbon/react";
import type { Meta, StoryObj } from "@storybook/nextjs-vite";

import { LocalDateTime } from "./local-date-time";

/** O instante, em ISO, `days` dias a partir de hoje, às `hour`:`minute` locais. */
function at(days: number, hour: number, minute = 0) {
  const date = new Date();
  date.setDate(date.getDate() + days);
  date.setHours(hour, minute, 0, 0);
  return date.toISOString();
}

const meta = {
  title: "Data e hora/LocalDateTime",
  component: LocalDateTime,
  args: { value: at(0, 21) },
  argTypes: { value: { control: "text" } },
} satisfies Meta<typeof LocalDateTime>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Today: Story = {};

export const Tomorrow: Story = { args: { value: at(1, 20, 30) } };

export const AnotherWeekday: Story = { args: { value: at(4, 19) } };

/** Em outro ano, o ano aparece. */
export const AnotherYear: Story = { args: { value: at(400, 21) } };

/** Com o fuso escrito, para o horário exato; no meio de uma frase, `LocalDayTime`. */
export const InSentence: Story = {
  render: (args) => (
    <p>
      O check-in fecha em <LocalDateTime {...args} />.
    </p>
  ),
};

export const Overview: Story = {
  render: () => (
    <Stack gap={3}>
      <LocalDateTime value={at(0, 21)} />
      <LocalDateTime value={at(1, 20, 30)} />
      <LocalDateTime value={at(4, 19)} />
      <LocalDateTime value={at(400, 21)} />
    </Stack>
  ),
};
