import type { Meta, StoryObj } from "@storybook/nextjs-vite";

import { LineChart, type LineChartPoint } from "./line-chart";

/** 40 mapas, um por dia, subindo e descendo em volta de 1500. */
function rating(count: number): LineChartPoint[] {
  let value = 1500;
  return Array.from({ length: count }, (_, i) => {
    value += i % 3 === 0 ? -24 : 31 - i;
    const date = new Date(Date.UTC(2026, 9, 1 + i, 21)).toISOString();
    return { group: "Rating", date, value };
  });
}

const meta = {
  title: "Estado e conteúdo/LineChart",
  component: LineChart,
  args: {
    points: rating(40),
    label: "Rating ao longo do tempo",
    valueTitle: "Rating",
  },
} satisfies Meta<typeof LineChart>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};

/** Poucos pontos: os primeiros mapas de alguém. */
export const FewPoints: Story = { args: { points: rating(3) } };

export const Phone: Story = {
  globals: { viewport: { value: "phone" } },
};
