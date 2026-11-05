import type { Meta, StoryObj } from "@storybook/nextjs-vite";

import { Stack } from "./carbon/stack";
import { TimeAgo } from "./time-ago";

/** O instante, em ISO, `seconds` segundos antes de agora. */
function ago(seconds: number) {
  return new Date(Date.now() - seconds * 1000).toISOString();
}

const meta = {
  title: "Data e hora/TimeAgo",
  component: TimeAgo,
  args: { value: ago(5 * 60) },
  argTypes: { value: { control: "text" } },
} satisfies Meta<typeof TimeAgo>;

export default meta;
type Story = StoryObj<typeof meta>;

/** Menos de um minuto: "agora". */
export const Seconds: Story = { args: { value: ago(10) } };

export const Minutes: Story = {};

export const Hours: Story = { args: { value: ago(3 * 3600) } };

/** Um dia atrás sai "ontem". */
export const Yesterday: Story = { args: { value: ago(86_400) } };

export const Days: Story = { args: { value: ago(4 * 86_400) } };

/** Refeito a cada 30 s com a aba aberta. */
export const Overview: Story = {
  render: () => (
    <Stack gap={3}>
      {[10, 5 * 60, 3 * 3600, 86_400, 4 * 86_400].map((seconds) => (
        <TimeAgo key={seconds} value={ago(seconds)} />
      ))}
    </Stack>
  ),
};
