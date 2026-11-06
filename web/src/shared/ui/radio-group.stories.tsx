import type { Meta, StoryObj } from "@storybook/nextjs-vite";
import { useState } from "react";
import { useArgs } from "storybook/preview-api";
import { fn } from "storybook/test";

import { Stack } from "./carbon/stack";
import { RadioGroup } from "./radio-group";

const options = [
  { value: "md1", label: "Série única" },
  { value: "md3", label: "Melhor de 3 (md3)" },
  { value: "md5", label: "Melhor de 5 (md5)" },
];

const meta = {
  title: "Formulário/RadioGroup",
  component: RadioGroup,
  args: {
    legendText: "Formato da série",
    name: "radio-group",
    options,
    value: "md3",
    onChange: fn(),
  },
  argTypes: {
    orientation: {
      control: "inline-radio",
      options: ["vertical", "horizontal"],
    },
    options: { control: "object" },
    value: { control: "text" },
    onChange: { control: false },
  },
  // Controlado: a escolha volta para os controles da história.
  render: function Render(args) {
    const [, updateArgs] = useArgs();
    return (
      <RadioGroup
        {...args}
        onChange={(value) => {
          args.onChange(value);
          updateArgs({ value });
        }}
      />
    );
  },
} satisfies Meta<typeof RadioGroup>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Vertical: Story = {};

/** Só para opções curtas; as que não cabem quebram de linha. */
export const Horizontal: Story = {
  args: {
    name: "radio-group-horizontal",
    orientation: "horizontal",
    options: [
      { value: "md1", label: "md1" },
      { value: "md3", label: "md3" },
      { value: "md5", label: "md5" },
    ],
  },
};

export const WithHelperText: Story = {
  args: {
    name: "radio-group-helper",
    helperText: "A final pode ter um formato próprio.",
  },
};

export const Invalid: Story = {
  args: {
    name: "radio-group-invalid",
    value: "md5",
    invalid: true,
    invalidText: "A md5 precisa de pelo menos 4 mapas.",
  },
};

export const Disabled: Story = {
  args: { name: "radio-group-disabled", disabled: true },
};

/** Só leitura: a escolha à vista, sem mudar (o torneio que já começou). */
export const ReadOnly: Story = {
  args: { name: "radio-group-readonly", readOnly: true },
};

export const OptionDisabled: Story = {
  args: {
    name: "radio-group-option-disabled",
    options: [
      { value: "md1", label: "Série única" },
      { value: "md3", label: "Melhor de 3 (md3)" },
      { value: "md5", label: "Melhor de 5 (md5)", disabled: true },
    ],
  },
};

function Stateful(props: {
  name: string;
  orientation?: "horizontal" | "vertical";
  disabled?: boolean;
  readOnly?: boolean;
  invalid?: boolean;
  invalidText?: string;
}) {
  const [value, setValue] = useState("md3");
  return (
    <RadioGroup
      {...props}
      legendText="Formato da série"
      options={options}
      value={value}
      onChange={setValue}
    />
  );
}

/** Os estados lado a lado. */
export const States: Story = {
  render: () => (
    <Stack gap={7}>
      <Stateful name="radio-group-states-default" />
      <Stateful name="radio-group-states-horizontal" orientation="horizontal" />
      <Stateful
        name="radio-group-states-invalid"
        invalid
        invalidText="A md5 precisa de pelo menos 4 mapas."
      />
      <Stateful name="radio-group-states-disabled" disabled />
      <Stateful name="radio-group-states-readonly" readOnly />
    </Stack>
  ),
};
