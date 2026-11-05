import type { Meta, StoryObj } from "@storybook/nextjs-vite";

import { AppFooter } from "./app-footer";

const meta = {
  title: "Página/AppFooter",
  component: AppFooter,
  parameters: {
    layout: "fullscreen",
    nextjs: { navigation: { pathname: "/pt/arena-brasil/copa-de-duelo" } },
  },
} satisfies Meta<typeof AppFooter>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};

/** Os links ganham a altura de toque. */
export const Phone: Story = {
  globals: { viewport: { value: "phone", isRotated: false } },
};
