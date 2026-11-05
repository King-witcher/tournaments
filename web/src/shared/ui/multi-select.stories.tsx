import type { Meta, StoryObj } from "@storybook/nextjs-vite";
import { fn } from "storybook/test";

import { Layer } from "./carbon/layer";
import { Stack } from "./carbon/stack";
import { MultiSelect } from "./multi-select";

const maps = [
  "Awoken",
  "Blood Covenant",
  "Blood Run",
  "Corrupted Keep",
  "Deep Embrace",
  "Insomnia",
  "Molten Falls",
  "Ruins of Sarnath",
  "Tempest Shrine",
  "Vale of Pnath",
].map((name) => ({ id: name.toLowerCase(), label: name }));

const allMaps = { id: "all", label: "Todos os mapas", isSelectAll: true };

const meta = {
  title: "Formulário/MultiSelect",
  component: MultiSelect,
  args: {
    id: "multi-select",
    titleText: "Mapas do torneio",
    label: "Escolha os mapas",
    items: maps,
    onChange: fn(),
  },
  argTypes: {
    size: { control: "inline-radio", options: ["sm", "md", "lg"] },
    type: { control: "inline-radio", options: ["default", "inline"] },
    direction: { control: "inline-radio", options: ["bottom", "top"] },
    selectionFeedback: {
      control: "inline-radio",
      options: ["top-after-reopen", "top", "fixed"],
    },
    items: { control: "object" },
    onChange: { control: false },
  },
} satisfies Meta<typeof MultiSelect>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};

/** O total escolhido fica na etiqueta, que limpa tudo. */
export const WithInitialSelectedItems: Story = {
  args: {
    id: "multi-select-initial",
    initialSelectedItems: [maps[0], maps[3], maps[7]],
  },
};

/** Lista longa: o campo de texto filtra, e `placeholder` ocupa o `label`. */
export const Filterable: Story = {
  args: {
    id: "multi-select-filterable",
    filterable: true,
    placeholder: "Digite para filtrar",
  },
};

/** O item com `isSelectAll` marca e desmarca todos. */
export const SelectAll: Story = {
  args: { id: "multi-select-all", items: [allMaps, ...maps] },
};

export const FilterableWithSelectAll: Story = {
  args: {
    id: "multi-select-filterable-all",
    filterable: true,
    placeholder: "Digite para filtrar",
    items: [allMaps, ...maps],
  },
};

export const WithHelperText: Story = {
  args: {
    id: "multi-select-helper",
    helperText: "O veto sai destes mapas.",
  },
};

/** Erro que veio da API (já traduzido). */
export const Invalid: Story = {
  args: {
    id: "multi-select-invalid",
    initialSelectedItems: [maps[0], maps[1]],
    invalid: true,
    invalidText: "A md5 precisa de pelo menos 5 mapas.",
  },
};

export const Warn: Story = {
  args: {
    id: "multi-select-warn",
    initialSelectedItems: [maps[0], maps[1], maps[2]],
    warn: true,
    warnText: "Com 3 mapas, o veto da md3 fica sem ban.",
  },
};

export const Disabled: Story = {
  args: {
    id: "multi-select-disabled",
    initialSelectedItems: [maps[0], maps[1]],
    disabled: true,
  },
};

export const ReadOnly: Story = {
  args: {
    id: "multi-select-readonly",
    initialSelectedItems: [maps[0], maps[1]],
    readOnly: true,
  },
};

/** Numa barra, ao lado do que filtra: o rótulo fica na mesma linha. */
export const Inline: Story = {
  args: { id: "multi-select-inline", type: "inline", titleText: "Mapas" },
};

/** Sobre um `Layer`, o campo troca o fundo para continuar visível. */
export const InLayer: Story = {
  args: { id: "multi-select-layer" },
  render: (args) => (
    <Layer>
      <MultiSelect {...args} />
    </Layer>
  ),
};

/** Os estados lado a lado. */
export const States: Story = {
  render: (args) => (
    <Stack gap={6}>
      <MultiSelect {...args} id="multi-select-states-default" />
      <MultiSelect
        {...args}
        id="multi-select-states-filterable"
        filterable
        placeholder="Digite para filtrar"
      />
      <MultiSelect
        {...args}
        id="multi-select-states-invalid"
        initialSelectedItems={[maps[0], maps[1]]}
        invalid
        invalidText="A md5 precisa de pelo menos 5 mapas."
      />
      <MultiSelect
        {...args}
        id="multi-select-states-warn"
        initialSelectedItems={[maps[0], maps[1], maps[2]]}
        warn
        warnText="Com 3 mapas, o veto da md3 fica sem ban."
      />
      <MultiSelect
        {...args}
        id="multi-select-states-disabled"
        initialSelectedItems={[maps[0], maps[1]]}
        disabled
      />
      <MultiSelect
        {...args}
        id="multi-select-states-readonly"
        initialSelectedItems={[maps[0], maps[1]]}
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
        <MultiSelect
          key={size}
          {...args}
          id={`multi-select-size-${size}`}
          titleText={size}
          size={size}
        />
      ))}
    </Stack>
  ),
};
