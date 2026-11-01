import { Layer, Stack } from "@carbon/react";
import type { Meta, StoryObj } from "@storybook/nextjs-vite";
import { useState } from "react";
import { useArgs } from "storybook/preview-api";
import { fn } from "storybook/test";

import { TimePicker } from "./time-picker";

const meta = {
  title: "Formulário/TimePicker",
  component: TimePicker,
  args: { id: "time-picker", value: "", onChange: fn() },
  argTypes: {
    size: { control: "inline-radio", options: ["sm", "md", "lg"] },
    onChange: { control: false },
    onTimeZoneChange: { control: false },
  },
  // Controlado: a hora e o fuso voltam para os controles da história.
  render: function Render(args) {
    const [, updateArgs] = useArgs();
    return (
      <TimePicker
        {...args}
        onChange={(value) => {
          args.onChange(value);
          updateArgs({ value });
        }}
        onTimeZoneChange={(timeZone) => updateArgs({ timeZone })}
      />
    );
  },
} satisfies Meta<typeof TimePicker>;

export default meta;
type Story = StoryObj<typeof meta>;

/** Só a hora, para quando o fuso já está dito na tela. */
export const Default: Story = {};

/** Com o seletor de fuso, que o Carbon pede para toda hora marcada. */
export const WithTimeZone: Story = {
  args: {
    id: "time-picker-zone",
    value: "21:00",
    timeZone: "America/Sao_Paulo",
  },
};

export const Required: Story = {
  args: { id: "time-picker-required", required: true },
};

/** Fora de 00:00 a 23:59 fica vermelho depois que a pessoa sai do campo. */
export const InvalidFormat: Story = {
  args: { id: "time-picker-format", value: "25:90" },
};

/** Erro que veio da API (já traduzido). */
export const Invalid: Story = {
  args: {
    id: "time-picker-invalid",
    value: "21:00",
    invalid: true,
    invalidText: "Outra série já começa neste horário.",
  },
};

export const Warning: Story = {
  args: {
    id: "time-picker-warning",
    value: "03:00",
    warning: true,
    warningText: "De madrugada pode faltar gente.",
  },
};

export const Disabled: Story = {
  args: {
    id: "time-picker-disabled",
    value: "21:00",
    timeZone: "America/Sao_Paulo",
    disabled: true,
  },
};

export const ReadOnly: Story = {
  args: {
    id: "time-picker-readonly",
    value: "21:00",
    timeZone: "America/Sao_Paulo",
    readOnly: true,
  },
};

function Stateful(
  props: Omit<
    Parameters<typeof TimePicker>[0],
    "value" | "onChange" | "onTimeZoneChange"
  >,
) {
  const [value, setValue] = useState("21:00");
  const [timeZone, setTimeZone] = useState(props.timeZone);
  return (
    <TimePicker
      {...props}
      value={value}
      onChange={setValue}
      timeZone={timeZone}
      onTimeZoneChange={setTimeZone}
    />
  );
}

/** Os três tamanhos (no celular, todos valem 48px). */
export const Sizes: Story = {
  render: () => (
    <Stack gap={6}>
      {(["sm", "md", "lg"] as const).map((size) => (
        <Stateful
          key={size}
          id={`time-picker-size-${size}`}
          labelText={`Tamanho ${size}`}
          size={size}
          timeZone="America/Sao_Paulo"
        />
      ))}
    </Stack>
  ),
};

/** Sobre uma camada (`Layer`), o campo troca de cor. */
export const WithLayer: Story = {
  render: () => (
    <Stack gap={5}>
      <Stateful id="time-picker-layer-0" timeZone="America/Sao_Paulo" />
      <Layer>
        <Stateful id="time-picker-layer-1" timeZone="America/Sao_Paulo" />
      </Layer>
    </Stack>
  ),
};
