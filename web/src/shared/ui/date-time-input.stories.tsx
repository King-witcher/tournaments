import { Stack } from "@carbon/react";
import type { Meta, StoryObj } from "@storybook/nextjs-vite";
import { useState } from "react";
import { useArgs } from "storybook/preview-api";
import { fn } from "storybook/test";

import { DateTimeInput } from "./date-time-input";

const meta = {
  title: "Formulário/DateTimeInput",
  component: DateTimeInput,
  args: {
    id: "date-time-input",
    labelText: "Início do torneio",
    value: null,
    onChange: fn(),
  },
  argTypes: {
    size: { control: "inline-radio", options: ["sm", "md", "lg"] },
    value: { control: "text" },
    onChange: { control: false },
    onBlur: { control: false },
  },
  // Controlado: o valor escolhido volta para os controles da história.
  render: function Render(args) {
    const [, updateArgs] = useArgs();
    return (
      <DateTimeInput
        {...args}
        onChange={(value) => {
          args.onChange(value);
          updateArgs({ value });
        }}
      />
    );
  },
} satisfies Meta<typeof DateTimeInput>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};

/** O valor entra em ISO (UTC) e aparece no fuso de quem usa. */
export const WithValue: Story = {
  args: { id: "date-time-input-value", value: "2026-11-14T21:00:00.000Z" },
};

export const WithHelperText: Story = {
  args: {
    id: "date-time-input-helper",
    helperText: "No seu fuso. As inscrições fecham uma hora antes.",
  },
};

export const Required: Story = {
  args: { id: "date-time-input-required", required: true },
};

/** Regra que só o chamador sabe (data à frente de agora): aparece na hora. */
export const Problem: Story = {
  args: {
    id: "date-time-input-problem",
    value: "2020-01-01T21:00:00.000Z",
    problem: "Escolha uma data no futuro.",
  },
};

/** Erro que veio da API (já traduzido). */
export const Invalid: Story = {
  args: {
    id: "date-time-input-invalid",
    value: "2026-11-14T21:00:00.000Z",
    invalid: true,
    invalidText: "Já existe um torneio neste horário.",
  },
};

export const Disabled: Story = {
  args: {
    id: "date-time-input-disabled",
    value: "2026-11-14T21:00:00.000Z",
    disabled: true,
  },
};

export const ReadOnly: Story = {
  args: {
    id: "date-time-input-readonly",
    value: "2026-11-14T21:00:00.000Z",
    readOnly: true,
  },
};

function Stateful(
  props: Omit<Parameters<typeof DateTimeInput>[0], "onChange">,
) {
  const [value, setValue] = useState(props.value);
  return <DateTimeInput {...props} value={value} onChange={setValue} />;
}

/** Os estados lado a lado. */
export const States: Story = {
  render: () => (
    <Stack gap={6}>
      <Stateful
        id="date-time-input-states-default"
        labelText="Início do torneio"
        value="2026-11-14T21:00:00.000Z"
      />
      <Stateful
        id="date-time-input-states-required"
        labelText="Início do torneio"
        value={null}
        required
      />
      <Stateful
        id="date-time-input-states-problem"
        labelText="Início do torneio"
        value="2020-01-01T21:00:00.000Z"
        problem="Escolha uma data no futuro."
      />
      <Stateful
        id="date-time-input-states-disabled"
        labelText="Início do torneio"
        value="2026-11-14T21:00:00.000Z"
        disabled
      />
      <Stateful
        id="date-time-input-states-readonly"
        labelText="Início do torneio"
        value="2026-11-14T21:00:00.000Z"
        readOnly
      />
    </Stack>
  ),
};
