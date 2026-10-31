import type { Meta, StoryObj } from "@storybook/nextjs-vite";
import { fn } from "storybook/test";

import { Countdown } from "./countdown";

/** O instante, em ISO, `seconds` segundos a partir de agora. */
function after(seconds: number) {
  return new Date(Date.now() + seconds * 1000).toISOString();
}

const meta = {
  title: "Data e hora/Countdown",
  component: Countdown,
  args: { until: after(12 * 60), onExpire: fn() },
  argTypes: { until: { control: "text" }, onExpire: { control: false } },
} satisfies Meta<typeof Countdown>;

export default meta;
type Story = StoryObj<typeof meta>;

/** Em minutos, `m:ss`. */
export const Minutes: Story = {};

/** Com mais de uma hora, `h:mm:ss`. */
export const Hours: Story = { args: { until: after(2 * 3600 + 15 * 60) } };

/** Conta o fim: ao chegar a zero chama `onExpire` uma vez (veja em Actions). */
export const Ending: Story = {
  // Calculado ao abrir a história, para não vencer antes de ser vista.
  render: (args) => <Countdown {...args} until={after(8)} />,
};

/** Passou do prazo: fica em `0:00` e chama `onExpire` ao montar. */
export const Expired: Story = { args: { until: after(-60) } };
