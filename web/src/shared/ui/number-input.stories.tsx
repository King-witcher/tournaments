import { Stack } from "@carbon/react";
import type { Meta, StoryObj } from "@storybook/nextjs-vite";
import { useState } from "react";
import { useArgs } from "storybook/preview-api";
import { fn } from "storybook/test";

import { NumberInput } from "./number-input";

const meta = {
  title: "Formulário/NumberInput",
  component: NumberInput,
  args: {
    id: "number-input",
    labelText: "Vagas",
    value: 16,
    min: 2,
    max: 64,
    onChange: fn(),
  },
  argTypes: {
    size: { control: "inline-radio", options: ["sm", "md", "lg"] },
    value: { control: "number" },
    onChange: { control: false },
  },
  // Controlado: o valor digitado volta para os controles da história.
  render: function Render(args) {
    const [, updateArgs] = useArgs();
    return (
      <NumberInput
        {...args}
        onChange={(value) => {
          args.onChange(value);
          updateArgs({ value });
        }}
      />
    );
  },
} satisfies Meta<typeof NumberInput>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};

export const WithHelperText: Story = {
  args: {
    id: "number-input-helper",
    helperText: "Conta quem entra pela lista de espera.",
  },
};

export const Required: Story = {
  args: { id: "number-input-required", required: true },
};

/** Fora de `min`/`max` fica vermelho na hora, com a faixa no erro. */
export const OutOfRange: Story = {
  args: { id: "number-input-range", value: 100 },
};

/** Só `min` ou só `max`: o erro diz "pelo menos" ou "no máximo". */
export const AtLeast: Story = {
  args: { id: "number-input-atleast", value: 1, min: 2, max: undefined },
};

/** O `invalidText` troca a mensagem da faixa. */
export const CustomInvalidText: Story = {
  args: {
    id: "number-input-custom",
    value: 100,
    invalidText: "Cabem no máximo 64 jogadores.",
  },
};

export const Warn: Story = {
  args: {
    id: "number-input-warn",
    value: 48,
    warn: true,
    warnText: "Com tantas vagas, o torneio dura o dia todo.",
  },
};

export const Disabled: Story = {
  args: { id: "number-input-disabled", disabled: true },
};

export const ReadOnly: Story = {
  args: { id: "number-input-readonly", readOnly: true },
};

export const HideSteppers: Story = {
  args: { id: "number-input-nosteppers", hideSteppers: true },
};

/** Aceita negativo: o `-` entra e o celular mostra o teclado com o sinal. */
export const Negative: Story = {
  args: {
    id: "number-input-negative",
    labelText: "Frags",
    value: -5,
    min: -999,
    max: 999,
    negative: true,
  },
};

/** Com `allowEmpty` o campo pode ficar vazio, e o valor é `null`. */
export const Empty: Story = {
  args: {
    id: "number-input-empty",
    labelText: "Limite de frags",
    value: null,
    allowEmpty: true,
    min: 1,
    max: 100,
  },
};

function Stateful(props: Omit<Parameters<typeof NumberInput>[0], "onChange">) {
  const [value, setValue] = useState(props.value);
  return <NumberInput {...props} value={value} onChange={setValue} />;
}

/** Os estados lado a lado. */
export const States: Story = {
  render: () => (
    <Stack gap={6}>
      <Stateful id="number-input-states-default" labelText="Vagas" value={16} />
      <Stateful
        id="number-input-states-required"
        labelText="Vagas"
        value={16}
        required
        min={2}
        max={64}
      />
      <Stateful
        id="number-input-states-invalid"
        labelText="Vagas"
        value={100}
        min={2}
        max={64}
      />
      <Stateful
        id="number-input-states-warn"
        labelText="Vagas"
        value={48}
        warn
        warnText="Com tantas vagas, o torneio dura o dia todo."
      />
      <Stateful
        id="number-input-states-disabled"
        labelText="Vagas"
        value={16}
        disabled
      />
      <Stateful
        id="number-input-states-readonly"
        labelText="Vagas"
        value={16}
        readOnly
      />
    </Stack>
  ),
};

/** Abaixo de `md` (celular), todos ficam com 48px de altura. */
export const Sizes: Story = {
  render: () => (
    <Stack gap={6}>
      {(["sm", "md", "lg"] as const).map((size) => (
        <Stateful
          key={size}
          id={`number-input-size-${size}`}
          labelText={size}
          value={16}
          size={size}
        />
      ))}
    </Stack>
  ),
};
