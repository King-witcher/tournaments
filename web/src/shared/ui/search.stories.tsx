import { Layer, SearchSkeleton, Stack } from "@carbon/react";
import type { Meta, StoryObj } from "@storybook/nextjs-vite";
import { fn } from "storybook/test";

import { Search } from "./search";

const meta = {
  title: "Formulário/Search",
  component: Search,
  args: {
    id: "search",
    labelText: "Buscar torneios",
    onChange: fn(),
    onClear: fn(),
  },
  argTypes: {
    size: { control: "inline-radio", options: ["sm", "md", "lg"] },
    onChange: { control: false },
    onClear: { control: false },
    onExpand: { control: false },
    renderIcon: { control: false },
  },
} satisfies Meta<typeof Search>;

export default meta;
type Story = StoryObj<typeof meta>;

/** O "Buscar" padrão no `placeholder`; o "x" aparece com texto. */
export const Default: Story = {};

export const WithValue: Story = {
  args: { id: "search-value", defaultValue: "Copa" },
};

export const CustomPlaceholder: Story = {
  args: { id: "search-placeholder", placeholder: "Nome ou comunidade" },
};

/** Só a lupa, que abre o campo ao clicar. */
export const Expandable: Story = {
  args: { id: "search-expandable", expandable: true },
};

/** Expansível que já começa aberto (e não fecha ao sair). */
export const ExpandableExpanded: Story = {
  args: { id: "search-expanded", expandable: true, isExpanded: true },
};

export const Disabled: Story = {
  args: { id: "search-disabled", disabled: true },
};

/** Sobre um `Layer`, o campo troca o fundo para continuar visível. */
export const InLayer: Story = {
  args: { id: "search-layer" },
  render: (args) => (
    <Layer>
      <Stack gap={5}>
        <Search {...args} />
        <Search {...args} id="search-layer-expandable" expandable />
      </Stack>
    </Layer>
  ),
};

/** No `loading.tsx`, com a forma do campo. */
export const Skeleton: Story = {
  render: () => (
    <div aria-hidden>
      <SearchSkeleton />
    </div>
  ),
};

/** Os estados lado a lado. */
export const States: Story = {
  render: (args) => (
    <Stack gap={5}>
      <Search {...args} id="search-states-default" />
      <Search {...args} id="search-states-value" defaultValue="Copa" />
      <Search {...args} id="search-states-expandable" expandable />
      <Search {...args} id="search-states-disabled" disabled />
    </Stack>
  ),
};

/** Abaixo de `md` (celular), `sm` e `md` ficam com 48px de altura. */
export const Sizes: Story = {
  render: (args) => (
    <Stack gap={5}>
      {(["sm", "md", "lg"] as const).map((size) => (
        <Search key={size} {...args} id={`search-size-${size}`} size={size} />
      ))}
    </Stack>
  ),
};
