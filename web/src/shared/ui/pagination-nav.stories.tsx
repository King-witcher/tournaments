import { Stack } from "@carbon/react";
import type { Meta, StoryObj } from "@storybook/nextjs-vite";
import { fn } from "storybook/test";

import { PaginationNav } from "./pagination-nav";

const meta = {
  title: "Listas/PaginationNav",
  component: PaginationNav,
  args: { totalItems: 25, page: 0, itemsShown: 10, onChange: fn() },
  argTypes: {
    size: { control: "inline-radio", options: ["sm", "md", "lg"] },
    tooltipAlignment: {
      control: "inline-radio",
      options: ["start", "center", "end"],
    },
    tooltipPosition: {
      control: "inline-radio",
      options: ["top", "right", "bottom", "left"],
    },
  },
} satisfies Meta<typeof PaginationNav>;

export default meta;
type Story = StoryObj<typeof meta>;

/** 25 páginas: as do meio ficam no "…", que abre um select com elas. */
export const Default: Story = {};

/** Numa página do meio: um "…" de cada lado. */
export const MiddlePage: Story = { args: { page: 12 } };

/** Poucas páginas: todas à vista, sem "…". */
export const FewPages: Story = { args: { totalItems: 5 } };

/** Da última volta para a primeira (e vice-versa) pelas setas. */
export const Loop: Story = { args: { loop: true } };

/** O "…" sem o select (para muitas páginas, quando montar a lista pesa). */
export const WithoutOverflowSelect: Story = {
  args: { disableOverflow: true, page: 12 },
};

/** Abaixo de `md` (celular), `sm` e `md` ficam com 48px de altura. */
export const Sizes: Story = {
  render: (args) => (
    <Stack gap={5}>
      {(["sm", "md", "lg"] as const).map((size) => (
        <PaginationNav key={size} {...args} size={size} />
      ))}
    </Stack>
  ),
};

/** No celular, no máximo 5 páginas à vista, para não rolar para o lado. */
export const Phone: Story = {
  globals: { viewport: { value: "phone", isRotated: false } },
};
