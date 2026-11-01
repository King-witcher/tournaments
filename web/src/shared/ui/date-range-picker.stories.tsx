import { DatePickerSkeleton, Layer } from "@carbon/react";
import type { Meta, StoryObj } from "@storybook/nextjs-vite";
import { useArgs } from "storybook/preview-api";
import { fn } from "storybook/test";

import { DateRangePicker } from "./date-range-picker";

const meta = {
  title: "Formulário/DateRangePicker",
  component: DateRangePicker,
  args: {
    id: "date-range-picker",
    startLabelText: "Início das inscrições",
    endLabelText: "Fim das inscrições",
    value: [null, null],
    onChange: fn(),
  },
  argTypes: {
    size: { control: "inline-radio", options: ["sm", "md", "lg"] },
    value: { control: "object" },
    onChange: { control: false },
  },
  // Controlado: o período escolhido volta para os controles da história.
  render: function Render(args) {
    const [, updateArgs] = useArgs();
    return (
      <DateRangePicker
        {...args}
        onChange={(value) => {
          args.onChange(value);
          updateArgs({ value });
        }}
      />
    );
  },
  // O calendário abre embaixo dos campos.
  parameters: { docs: { story: { inline: false, height: "420px" } } },
} satisfies Meta<typeof DateRangePicker>;

export default meta;
type Story = StoryObj<typeof meta>;

export const RangeWithCalendar: Story = {};

export const WithValue: Story = {
  args: { id: "date-range-picker-value", value: ["2026-11-09", "2026-11-14"] },
};

/** Fora da faixa, o dia fica desabilitado no calendário. */
export const MinAndMax: Story = {
  args: {
    id: "date-range-picker-min-max",
    minDate: "2026-11-01",
    maxDate: "2026-11-30",
  },
};

export const Required: Story = {
  args: { id: "date-range-picker-required", required: true },
};

/** Erro que veio da API (já traduzido): o Carbon marca as duas pontas. */
export const Invalid: Story = {
  args: {
    id: "date-range-picker-invalid",
    value: ["2026-11-09", "2026-11-14"],
    invalid: true,
    invalidText: "As inscrições precisam fechar antes do início.",
  },
};

export const Warning: Story = {
  args: {
    id: "date-range-picker-warn",
    value: ["2026-11-09", "2026-11-10"],
    warn: true,
    warnText: "Dois dias é pouco para encher a chave.",
  },
};

export const Disabled: Story = {
  args: {
    id: "date-range-picker-disabled",
    value: ["2026-11-09", "2026-11-14"],
    disabled: true,
  },
};

export const ReadOnly: Story = {
  args: {
    id: "date-range-picker-readonly",
    value: ["2026-11-09", "2026-11-14"],
    readOnly: true,
  },
};

/** Sobre uma camada (`Layer`), os campos trocam de cor. */
export const WithLayer: Story = {
  render: function Render(args) {
    const [, updateArgs] = useArgs();
    return (
      <Layer>
        <DateRangePicker
          {...args}
          id="date-range-picker-layer"
          onChange={(value) => updateArgs({ value })}
        />
      </Layer>
    );
  },
};

/** O esqueleto do Carbon, para o `loading.tsx`. */
export const Skeleton: Story = {
  render: () => <DatePickerSkeleton range />,
};
