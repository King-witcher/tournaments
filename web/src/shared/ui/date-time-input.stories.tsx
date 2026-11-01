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
    onTimeZoneChange: { control: false },
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
  // O calendário abre embaixo do campo.
  parameters: { docs: { story: { inline: false, height: "420px" } } },
} satisfies Meta<typeof DateTimeInput>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};

/** O valor entra em ISO (UTC) e aparece no fuso do seletor. */
export const WithValue: Story = {
  args: { id: "date-time-input-value", value: "2026-11-14T21:00:00.000Z" },
};

/** Outra dica no lugar do formato (que fica no placeholder). */
export const WithHelperText: Story = {
  args: {
    id: "date-time-input-helper",
    value: "2026-11-14T21:00:00.000Z",
    helperText: "O check-in abre sáb., 14 de nov., 17:30.",
  },
};

/** Os dias antes do mínimo saem do calendário. */
export const WithMin: Story = {
  args: { id: "date-time-input-min", min: new Date().toISOString() },
};

export const Required: Story = {
  args: { id: "date-time-input-required", required: true },
};

/** Só a data: o valor continua `null` e a hora passa a ser obrigatória. */
export const DateOnly: Story = {
  render: function Render(args) {
    const [value, setValue] = useState<string | null>(null);
    return (
      <DateTimeInput
        {...args}
        id="date-time-input-date-only"
        value={value}
        onChange={(next) => {
          args.onChange(next);
          setValue(next);
        }}
      />
    );
  },
};

/** Regra que só o chamador sabe (data à frente de agora): aparece na hora. */
export const Problem: Story = {
  args: {
    id: "date-time-input-problem",
    value: "2020-01-01T21:00:00.000Z",
    problem: "A data de início já passou.",
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

/**
 * O fuso controlado de fora: quem escreve outra data na tela (quando o
 * check-in abre) usa o mesmo fuso. Trocar o fuso mantém o relógio e muda o
 * instante.
 */
export const ControlledTimeZone: Story = {
  render: function Render(args) {
    const [value, setValue] = useState<string | null>(
      "2026-11-14T21:00:00.000Z",
    );
    const [timeZone, setTimeZone] = useState("Europe/Madrid");
    return (
      <Stack gap={5}>
        <DateTimeInput
          {...args}
          id="date-time-input-zone"
          value={value}
          onChange={setValue}
          timeZone={timeZone}
          onTimeZoneChange={setTimeZone}
        />
        <p>Valor: {value ?? "—"}</p>
      </Stack>
    );
  },
};

/** No celular, a hora desce para baixo da data. */
export const Phone: Story = {
  args: { id: "date-time-input-phone", value: "2026-11-14T21:00:00.000Z" },
  globals: { viewport: { value: "phone" } },
};
