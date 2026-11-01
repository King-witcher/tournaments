import { Layer, SliderSkeleton, Stack } from "@carbon/react";
import type { Meta, StoryObj } from "@storybook/nextjs-vite";
import { useArgs } from "storybook/preview-api";
import { fn } from "storybook/test";

import { Slider } from "./slider";

const range = {
  labelText: "Faixa de rating",
  min: 0,
  max: 3000,
  step: 50,
  stepMultiplier: 10,
  value: 1200,
  unstable_valueUpper: 1800,
};

const meta = {
  title: "Formulário/Slider",
  component: Slider,
  args: {
    id: "slider",
    labelText: "Limite de frags",
    min: 5,
    max: 50,
    step: 5,
    value: 15,
    onChange: fn(),
  },
  argTypes: {
    value: { control: "number" },
    unstable_valueUpper: { control: "number" },
    onChange: { control: false },
    onRelease: { control: false },
    onBlur: { control: false },
    formatLabel: { control: false },
  },
  // Controlado: o valor arrastado volta para os controles da história.
  render: function Render(args) {
    const [, updateArgs] = useArgs();
    return (
      <Slider
        {...args}
        onChange={(data) => {
          args.onChange?.(data);
          updateArgs(
            data.valueUpper === undefined
              ? { value: data.value }
              : { value: data.value, unstable_valueUpper: data.valueUpper },
          );
        }}
      />
    );
  },
} satisfies Meta<typeof Slider>;

export default meta;
type Story = StoryObj<typeof meta>;

/** A barra e o campo para digitar o número exato. */
export const Default: Story = {};

/** Sem o campo: o valor aparece na dica da alça. */
export const HiddenInput: Story = {
  args: { id: "slider-hidden", hideTextInput: true },
};

/** `formatLabel` escreve os extremos e a dica com a unidade. */
export const CustomValueLabel: Story = {
  args: {
    id: "slider-label",
    hideTextInput: true,
    formatLabel: (value) => `${value} frags`,
  },
};

/** Duas alças: de `value` a `unstable_valueUpper`. */
export const TwoHandles: Story = {
  args: { id: "slider-range", ...range },
};

export const TwoHandlesHiddenInputs: Story = {
  args: { id: "slider-range-hidden", ...range, hideTextInput: true },
};

export const Required: Story = {
  args: { id: "slider-required", required: true },
};

/** Erro que veio da API (já traduzido). */
export const Invalid: Story = {
  args: {
    id: "slider-invalid",
    value: 50,
    invalid: true,
    invalidText: "No FFA, o limite é de 30 frags.",
  },
};

export const Warn: Story = {
  args: {
    id: "slider-warn",
    value: 45,
    warn: true,
    warnText: "Com mais de 40 frags, a partida passa de 15 minutos.",
  },
};

export const Disabled: Story = {
  args: { id: "slider-disabled", disabled: true },
};

export const ReadOnly: Story = {
  args: { id: "slider-readonly", readOnly: true },
};

/** Sobre um `Layer`, o campo troca o fundo para continuar visível. */
export const InLayer: Story = {
  args: { id: "slider-layer" },
  render: (args) => (
    <Layer>
      <Slider {...args} />
    </Layer>
  ),
};

/** No `loading.tsx`, com a forma do campo. */
export const Skeleton: Story = {
  render: () => (
    <div aria-hidden>
      <SliderSkeleton />
    </div>
  ),
};

export const TwoHandlesSkeleton: Story = {
  render: () => (
    <div aria-hidden>
      <SliderSkeleton twoHandles />
    </div>
  ),
};

/** Os estados lado a lado (sem guardar o valor arrastado). */
export const States: Story = {
  render: (args) => (
    <Stack gap={7}>
      <Slider {...args} id="slider-states-default" />
      <Slider {...args} id="slider-states-range" {...range} />
      <Slider {...args} id="slider-states-required" required />
      <Slider
        {...args}
        id="slider-states-invalid"
        value={50}
        invalid
        invalidText="No FFA, o limite é de 30 frags."
      />
      <Slider
        {...args}
        id="slider-states-warn"
        value={45}
        warn
        warnText="Com mais de 40 frags, a partida passa de 15 minutos."
      />
      <Slider {...args} id="slider-states-disabled" disabled />
      <Slider {...args} id="slider-states-readonly" readOnly />
    </Stack>
  ),
};
