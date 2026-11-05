import type { Meta, StoryObj } from "@storybook/nextjs-vite";
import { useState } from "react";
import { useArgs } from "storybook/preview-api";
import { fn } from "storybook/test";

import { DatePickerSkeleton } from "./carbon/date-picker";
import { Layer } from "./carbon/layer";
import { Stack } from "./carbon/stack";
import { DatePicker } from "./date-picker";

const meta = {
  title: "Formulário/DatePicker",
  component: DatePicker,
  args: {
    id: "date-picker",
    labelText: "Data da final",
    value: null,
    onChange: fn(),
  },
  argTypes: {
    datePickerType: { control: "inline-radio", options: ["single", "simple"] },
    size: { control: "inline-radio", options: ["sm", "md", "lg"] },
    value: { control: "text" },
    onChange: { control: false },
  },
  // Controlado: a data escolhida volta para os controles da história.
  render: function Render(args) {
    const [, updateArgs] = useArgs();
    return (
      <DatePicker
        {...args}
        onChange={(value) => {
          args.onChange(value);
          updateArgs({ value });
        }}
      />
    );
  },
  // O calendário abre embaixo do campo.
  parameters: { docs: { story: { inline: false, height: "420px" } } },
} satisfies Meta<typeof DatePicker>;

export default meta;
type Story = StoryObj<typeof meta>;

/** Com o calendário: para a data que a pessoa precisa ver perto das outras. */
export const SingleWithCalendar: Story = {};

/** Só o campo: para a data que se sabe de cor. */
export const Simple: Story = {
  args: {
    id: "date-picker-simple",
    labelText: "Data de nascimento",
    datePickerType: "simple",
  },
};

export const WithValue: Story = {
  args: { id: "date-picker-value", value: "2026-11-14" },
};

/** Fora da faixa, o dia fica desabilitado no calendário. */
export const MinAndMax: Story = {
  args: {
    id: "date-picker-min-max",
    value: "2026-11-14",
    minDate: "2026-11-09",
    maxDate: "2026-11-22",
  },
};

/** Outra dica no lugar do formato (que fica no placeholder). */
export const WithHelperText: Story = {
  args: {
    id: "date-picker-helper",
    helperText: "As inscrições fecham na véspera.",
  },
};

export const Required: Story = {
  args: { id: "date-picker-required", required: true },
};

/** Regra que só o chamador sabe: aparece na hora e bloqueia o envio. */
export const Problem: Story = {
  args: {
    id: "date-picker-problem",
    value: "2020-01-01",
    problem: "A data já passou.",
  },
};

/** Erro que veio da API (já traduzido). */
export const Invalid: Story = {
  args: {
    id: "date-picker-invalid",
    value: "2026-11-14",
    invalid: true,
    invalidText: "Já existe um torneio nesta data.",
  },
};

export const Warning: Story = {
  args: {
    id: "date-picker-warn",
    value: "2026-12-25",
    warn: true,
    warnText: "É feriado: pode faltar gente.",
  },
};

export const Disabled: Story = {
  args: { id: "date-picker-disabled", value: "2026-11-14", disabled: true },
};

export const ReadOnly: Story = {
  args: { id: "date-picker-readonly", value: "2026-11-14", readOnly: true },
};

function Stateful(
  props: Omit<Parameters<typeof DatePicker>[0], "value" | "onChange">,
) {
  const [value, setValue] = useState<string | null>("2026-11-14");
  return <DatePicker {...props} value={value} onChange={setValue} />;
}

/** Os três tamanhos (no celular, todos valem 48px). */
export const Sizes: Story = {
  render: () => (
    <Stack gap={6}>
      {(["sm", "md", "lg"] as const).map((size) => (
        <Stateful
          key={size}
          id={`date-picker-size-${size}`}
          labelText={`Tamanho ${size}`}
          size={size}
        />
      ))}
    </Stack>
  ),
};

/** Sobre uma camada (`Layer`), o campo troca de cor para continuar à vista. */
export const WithLayer: Story = {
  render: () => (
    <Stack gap={5}>
      <Stateful id="date-picker-layer-0" labelText="Na página" />
      <Layer>
        <Stateful id="date-picker-layer-1" labelText="Na primeira camada" />
        <Layer>
          <Stateful id="date-picker-layer-2" labelText="Na segunda camada" />
        </Layer>
      </Layer>
    </Stack>
  ),
};

/** O esqueleto do Carbon, para o `loading.tsx`. */
export const Skeleton: Story = {
  render: () => <DatePickerSkeleton />,
};
