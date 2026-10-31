import { Stack } from "@carbon/react";
import type { Meta, StoryObj } from "@storybook/nextjs-vite";

import { LocalDate } from "./local-date";

/** O instante, em ISO, `days` dias a partir de hoje, ao meio-dia local. */
function at(days: number) {
  const date = new Date();
  date.setDate(date.getDate() + days);
  date.setHours(12, 0, 0, 0);
  return date.toISOString();
}

const meta = {
  title: "Data e hora/LocalDate",
  component: LocalDate,
  args: { value: at(0) },
  argTypes: { value: { control: "text" } },
} satisfies Meta<typeof LocalDate>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Today: Story = {};

export const Tomorrow: Story = { args: { value: at(1) } };

export const AWeekAgo: Story = { args: { value: at(-7) } };

/** O ano sempre aparece. */
export const AnotherYear: Story = { args: { value: at(-400) } };

export const InSentence: Story = {
  render: (args) => (
    <p>
      Membro da Arena Brasil desde <LocalDate {...args} />.
    </p>
  ),
  args: { value: at(-120) },
};

export const Overview: Story = {
  render: () => (
    <Stack gap={3}>
      <LocalDate value={at(0)} />
      <LocalDate value={at(1)} />
      <LocalDate value={at(-7)} />
      <LocalDate value={at(-400)} />
    </Stack>
  ),
};
