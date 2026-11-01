import { PaginationSkeleton, Stack } from "@carbon/react";
import type { Meta, StoryObj } from "@storybook/nextjs-vite";
import { fn } from "storybook/test";

import { Pagination } from "./pagination";

const meta = {
  title: "Listas/Pagination",
  component: Pagination,
  args: {
    totalItems: 103,
    pageSize: 25,
    pageSizes: [25, 50, 100],
    onChange: fn(),
  },
  argTypes: {
    size: { control: "inline-radio", options: ["xs", "sm", "md", "lg"] },
    pageSizes: { control: "object" },
  },
} satisfies Meta<typeof Pagination>;

export default meta;
type Story = StoryObj<typeof meta>;

/** "Por página", o intervalo, a página no select e as setas, traduzidos. */
export const Default: Story = {};

/** Começa numa página do meio: as duas setas ficam ativas. */
export const MiddlePage: Story = { args: { page: 3 } };

export const Disabled: Story = { args: { disabled: true } };

/** Abaixo de `md` (celular), `sm` e `md` ficam com 48px de altura. */
export const Sizes: Story = {
  render: (args) => (
    <Stack gap={5}>
      {(["xs", "sm", "md", "lg"] as const).map((size) => (
        <Pagination key={size} {...args} size={size} />
      ))}
    </Stack>
  ),
};

/** Opções do "por página" com texto próprio. */
export const CustomPageSizesLabel: Story = {
  args: {
    pageSize: 10,
    pageSizes: [
      { text: "Dez", value: 10 },
      { text: "Vinte", value: 20 },
      { text: "Trinta", value: 30 },
    ],
  },
};

/** Sem o total: só o intervalo e a página atual, sem o select de páginas. */
export const PagesUnknown: Story = {
  args: { pagesUnknown: true, totalItems: undefined },
};

/** Sem o "por página": o tamanho da página é fixo. */
export const WithoutPageSizes: Story = { args: { pageSizes: undefined } };

/** Duas paginações na mesma página: cada uma com os próprios ids. */
export const MultiplePaginationComponents: Story = {
  render: (args) => (
    <Stack gap={5}>
      <Pagination {...args} />
      <Pagination {...args} />
    </Stack>
  ),
};

/** Com menos de 42rem de largura, ficam só o intervalo e as setas. */
export const Phone: Story = {
  globals: { viewport: { value: "phone", isRotated: false } },
};

/** Carregando: `PaginationSkeleton` do `@carbon/react`. */
export const Skeleton: Story = { render: () => <PaginationSkeleton /> };
