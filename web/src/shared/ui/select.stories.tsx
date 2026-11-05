import type { Meta, StoryObj } from "@storybook/nextjs-vite";
import { fn } from "storybook/test";

import { SelectItem } from "./carbon/select";
import { Stack } from "./carbon/stack";
import { Select } from "./select";

const options = (
  <>
    <SelectItem value="" text="Escolha o formato" />
    <SelectItem value="duel" text="Duelo" />
    <SelectItem value="ffa" text="Todos contra todos" />
    <SelectItem value="2v2" text="2v2" />
    <SelectItem value="3v3" text="3v3" />
  </>
);

const meta = {
  title: "Formulário/Select",
  component: Select,
  args: {
    id: "select",
    labelText: "Formato",
    children: options,
    onChange: fn(),
  },
  argTypes: {
    size: { control: "inline-radio", options: ["sm", "md", "lg"] },
    children: { control: false },
    onChange: { control: false },
  },
} satisfies Meta<typeof Select>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};

export const WithHelperText: Story = {
  args: {
    id: "select-helper",
    helperText: "Dá para mudar até abrir as inscrições.",
  },
};

export const Required: Story = {
  args: { id: "select-required", required: true },
};

/** Erro que veio da API (já traduzido). */
export const Invalid: Story = {
  args: {
    id: "select-invalid",
    defaultValue: "3v3",
    invalid: true,
    invalidText: "Este formato precisa de pelo menos 6 vagas.",
  },
};

export const Warn: Story = {
  args: {
    id: "select-warn",
    defaultValue: "ffa",
    warn: true,
    warnText: "Com mais de 8 vagas, o formato fica longo.",
  },
};

export const Disabled: Story = {
  args: { id: "select-disabled", defaultValue: "duel", disabled: true },
};

export const ReadOnly: Story = {
  args: { id: "select-readonly", defaultValue: "duel", readOnly: true },
};

export const Inline: Story = {
  args: { id: "select-inline", inline: true, defaultValue: "duel" },
};

/** Os estados lado a lado. */
export const States: Story = {
  render: (args) => (
    <Stack gap={6}>
      <Select {...args} id="select-states-default" />
      <Select
        {...args}
        id="select-states-helper"
        helperText="Dá para mudar até abrir as inscrições."
      />
      <Select {...args} id="select-states-required" required />
      <Select
        {...args}
        id="select-states-invalid"
        defaultValue="3v3"
        invalid
        invalidText="Este formato precisa de pelo menos 6 vagas."
      />
      <Select
        {...args}
        id="select-states-warn"
        defaultValue="ffa"
        warn
        warnText="Com mais de 8 vagas, o formato fica longo."
      />
      <Select
        {...args}
        id="select-states-disabled"
        defaultValue="duel"
        disabled
      />
      <Select
        {...args}
        id="select-states-readonly"
        defaultValue="duel"
        readOnly
      />
    </Stack>
  ),
};

/** Abaixo de `md` (celular), todos ficam com 48px de altura. */
export const Sizes: Story = {
  render: (args) => (
    <Stack gap={6}>
      {(["sm", "md", "lg"] as const).map((size) => (
        <Select
          key={size}
          {...args}
          id={`select-size-${size}`}
          labelText={size}
          size={size}
        />
      ))}
    </Stack>
  ),
};
