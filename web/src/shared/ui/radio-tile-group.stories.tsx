import type { Meta, StoryObj } from "@storybook/nextjs-vite";
import { useState } from "react";
import { useArgs } from "storybook/preview-api";
import { fn } from "storybook/test";

import { Layer } from "./carbon/layer";
import { Stack } from "./carbon/stack";
import { RadioTileGroup } from "./radio-tile-group";

const formats = [
  { value: "single", content: "Eliminação simples" },
  { value: "double", content: "Eliminação dupla" },
  { value: "round-robin", content: "Todos contra todos" },
  { value: "swiss", content: "Suíço" },
];

/** Com o que cada opção faz, para comparar antes de escolher. */
const described = [
  {
    value: "single",
    content: "Eliminação simples",
    description: "Quem perde uma série está fora.",
  },
  {
    value: "double",
    content: "Eliminação dupla",
    description:
      "Quem perde cai para a repescagem e, se perder de novo, está fora.",
  },
  {
    value: "round-robin",
    content: "Todos contra todos",
    description: "Cada um enfrenta todos os outros uma vez.",
  },
];

const meta = {
  title: "Formulário/RadioTileGroup",
  component: RadioTileGroup,
  args: {
    legend: "Formato do torneio",
    name: "radio-tile-group",
    options: formats,
    value: "single",
    onChange: fn(),
  },
  argTypes: {
    options: { control: "object" },
    value: { control: "text" },
    onChange: { control: false },
  },
  // Controlado: a escolha volta para os controles da história.
  render: function Render(args) {
    const [, updateArgs] = useArgs();
    return (
      <RadioTileGroup
        {...args}
        onChange={(value) => {
          args.onChange(value);
          updateArgs({ value });
        }}
      />
    );
  },
} satisfies Meta<typeof RadioTileGroup>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};

/** Nada escolhido antes da primeira escolha. */
export const NoSelection: Story = {
  args: { name: "radio-tile-group-empty", value: undefined },
};

export const Required: Story = {
  args: {
    name: "radio-tile-group-required",
    value: undefined,
    required: true,
  },
};

export const Disabled: Story = {
  args: { name: "radio-tile-group-disabled", disabled: true },
};

export const OptionDisabled: Story = {
  args: {
    name: "radio-tile-group-option-disabled",
    options: [...formats.slice(0, 3), { ...formats[3], disabled: true }],
  },
};

export const WithDescription: Story = {
  args: { name: "radio-tile-group-described", options: described },
};

/** Só leitura: só a escolhida, num tile sem interação. */
export const ReadOnly: Story = {
  args: {
    name: "radio-tile-group-read-only",
    options: described,
    value: "double",
    readOnly: true,
  },
};

/** No celular, uma coluna, com o texto quebrando dentro do tile. */
export const Phone: Story = {
  args: { name: "radio-tile-group-phone", options: described },
  globals: { viewport: { value: "phone" } },
};

function Stateful({ name }: { name: string }) {
  const [value, setValue] = useState<string | undefined>("double");
  return (
    <RadioTileGroup
      legend="Formato do torneio"
      name={name}
      options={formats}
      value={value}
      onChange={setValue}
    />
  );
}

/** Sobre um `Layer`, os tiles trocam o fundo para continuar visíveis. */
export const InLayer: Story = {
  render: () => (
    <Layer>
      <Stateful name="radio-tile-group-layer" />
    </Layer>
  ),
};

/** Os estados lado a lado. */
export const States: Story = {
  render: () => (
    <Stack gap={7}>
      <Stateful name="radio-tile-group-states-default" />
      <RadioTileGroup
        legend="Formato do torneio"
        name="radio-tile-group-states-disabled"
        options={formats}
        value="single"
        onChange={fn()}
        disabled
      />
    </Stack>
  ),
};
