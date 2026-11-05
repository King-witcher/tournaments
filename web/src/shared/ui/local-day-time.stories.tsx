import type { Meta, StoryObj } from "@storybook/nextjs-vite";

import { Stack } from "./carbon/stack";
import { LocalDayTime } from "./local-day-time";

/** O instante, em ISO, `days` dias a partir de hoje, às `hour`:`minute` locais. */
function at(days: number, hour: number, minute = 0) {
  const date = new Date();
  date.setDate(date.getDate() + days);
  date.setHours(hour, minute, 0, 0);
  return date.toISOString();
}

/** Para o meio de uma frase, que não leva preposição própria. */
function inSentence(value: string, sameDayAs?: string) {
  return (
    <p>
      O torneio começa <LocalDayTime value={value} sameDayAs={sameDayAs} />.
    </p>
  );
}

const meta = {
  title: "Data e hora/LocalDayTime",
  component: LocalDayTime,
  args: { value: at(0, 21) },
  argTypes: { value: { control: "text" }, sameDayAs: { control: "text" } },
  render: (args) => inSentence(args.value, args.sameDayAs ?? undefined),
} satisfies Meta<typeof LocalDayTime>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Today: Story = {};

export const Tomorrow: Story = { args: { value: at(1, 20, 30) } };

export const Yesterday: Story = { args: { value: at(-1, 21) } };

/** Fora de ontem, hoje e amanhã, sai a data. */
export const AnotherDay: Story = { args: { value: at(4, 20) } };

/** Em outro ano, a data leva o ano. */
export const AnotherYear: Story = { args: { value: at(400, 20) } };

/** No mesmo dia que a frase anterior já situou, sai só a hora. */
export const SameDayAs: Story = {
  render: () => (
    <p>
      A semifinal é <LocalDayTime value={at(1, 20)} />, e a final,{" "}
      <LocalDayTime value={at(1, 22, 15)} sameDayAs={at(1, 20)} />.
    </p>
  ),
};

export const Overview: Story = {
  render: () => (
    <Stack gap={3}>
      {[at(-1, 21), at(0, 21), at(1, 20, 30), at(4, 20), at(400, 20)].map(
        (value) => (
          <div key={value}>{inSentence(value)}</div>
        ),
      )}
    </Stack>
  ),
};
