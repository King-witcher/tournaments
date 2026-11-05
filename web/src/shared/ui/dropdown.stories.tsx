import type { Meta, StoryObj } from "@storybook/nextjs-vite";
import { fn } from "storybook/test";

import { DropdownSkeleton } from "./carbon/dropdown";
import { Layer } from "./carbon/layer";
import { Stack } from "./carbon/stack";
import { Dropdown } from "./dropdown";

const items = [
  { id: "start", label: "Data de início" },
  { id: "name", label: "Nome" },
  { id: "players", label: "Inscritos" },
  { id: "status", label: "Situação" },
];

const meta = {
  title: "Formulário/Dropdown",
  component: Dropdown,
  args: {
    id: "dropdown",
    titleText: "Ordenar os torneios por",
    label: "Escolha a ordem",
    items,
    onChange: fn(),
  },
  argTypes: {
    size: { control: "inline-radio", options: ["sm", "md", "lg"] },
    type: { control: "inline-radio", options: ["default", "inline"] },
    direction: { control: "inline-radio", options: ["bottom", "top"] },
    items: { control: "object" },
    onChange: { control: false },
  },
} satisfies Meta<typeof Dropdown>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};

export const WithSelection: Story = {
  args: { id: "dropdown-selection", initialSelectedItem: items[0] },
};

export const WithHelperText: Story = {
  args: {
    id: "dropdown-helper",
    helperText: "Vale só para esta lista.",
  },
};

/** Erro que veio da API (já traduzido). */
export const Invalid: Story = {
  args: {
    id: "dropdown-invalid",
    initialSelectedItem: items[2],
    invalid: true,
    invalidText: "Não deu para ordenar por inscritos agora.",
  },
};

export const Warn: Story = {
  args: {
    id: "dropdown-warn",
    initialSelectedItem: items[3],
    warn: true,
    warnText: "Os encerrados ficam no fim.",
  },
};

export const Disabled: Story = {
  args: {
    id: "dropdown-disabled",
    initialSelectedItem: items[0],
    disabled: true,
  },
};

export const ReadOnly: Story = {
  args: {
    id: "dropdown-readonly",
    initialSelectedItem: items[0],
    readOnly: true,
  },
};

/** Item com `disabled` aparece no menu sem poder ser escolhido. */
export const DisabledItem: Story = {
  args: {
    id: "dropdown-disabled-item",
    items: [...items.slice(0, 3), { ...items[3], disabled: true }],
  },
};

/** Numa barra, ao lado do que filtra: o rótulo fica na mesma linha. */
export const Inline: Story = {
  args: {
    id: "dropdown-inline",
    type: "inline",
    titleText: "Ordem",
    initialSelectedItem: items[0],
  },
};

/** Sobre um `Layer`, o campo troca o fundo para continuar visível. */
export const InLayer: Story = {
  args: { id: "dropdown-layer" },
  render: (args) => (
    <Layer>
      <Dropdown {...args} />
    </Layer>
  ),
};

/** No `loading.tsx`, com a forma do campo. */
export const Skeleton: Story = {
  render: () => (
    <div aria-hidden>
      <DropdownSkeleton />
    </div>
  ),
};

/** Os estados lado a lado. */
export const States: Story = {
  render: (args) => (
    <Stack gap={6}>
      <Dropdown {...args} id="dropdown-states-default" />
      <Dropdown
        {...args}
        id="dropdown-states-helper"
        helperText="Vale só para esta lista."
      />
      <Dropdown
        {...args}
        id="dropdown-states-invalid"
        initialSelectedItem={items[2]}
        invalid
        invalidText="Não deu para ordenar por inscritos agora."
      />
      <Dropdown
        {...args}
        id="dropdown-states-warn"
        initialSelectedItem={items[3]}
        warn
        warnText="Os encerrados ficam no fim."
      />
      <Dropdown
        {...args}
        id="dropdown-states-disabled"
        initialSelectedItem={items[0]}
        disabled
      />
      <Dropdown
        {...args}
        id="dropdown-states-readonly"
        initialSelectedItem={items[0]}
        readOnly
      />
    </Stack>
  ),
};

/** Abaixo de `md` (celular), `sm` e `md` ficam com 48px de altura. */
export const Sizes: Story = {
  render: (args) => (
    <Stack gap={6}>
      {(["sm", "md", "lg"] as const).map((size) => (
        <Dropdown
          key={size}
          {...args}
          id={`dropdown-size-${size}`}
          titleText={size}
          size={size}
        />
      ))}
    </Stack>
  ),
};
