import type { Meta, StoryObj } from "@storybook/nextjs-vite";
import { fn } from "storybook/test";

import { Layer } from "./carbon/layer";
import { Stack } from "./carbon/stack";
import { ComboBox } from "./combo-box";

const champions = [
  "Anarki",
  "Athena",
  "B.J. Blazkowicz",
  "Clutch",
  "Death Knight",
  "Doom Slayer",
  "Eisen",
  "Galena",
  "Keel",
  "Nyx",
  "Ranger",
  "Scalebearer",
  "Slash",
  "Sorlag",
  "Strogg & Peeker",
  "Visor",
].map((name) => ({ id: name.toLowerCase(), label: name }));

const meta = {
  title: "Formulário/ComboBox",
  component: ComboBox,
  args: {
    id: "combo-box",
    titleText: "Campeão",
    placeholder: "Digite para filtrar",
    items: champions,
    onChange: fn(),
  },
  argTypes: {
    size: { control: "inline-radio", options: ["sm", "md", "lg"] },
    direction: { control: "inline-radio", options: ["bottom", "top"] },
    items: { control: "object" },
    onChange: { control: false },
  },
} satisfies Meta<typeof ComboBox>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};

export const WithSelection: Story = {
  args: { id: "combo-box-selection", initialSelectedItem: champions[9] },
};

export const WithHelperText: Story = {
  args: {
    id: "combo-box-helper",
    helperText: "O campeão que você mais usa no duelo.",
  },
};

export const Required: Story = {
  args: { id: "combo-box-required", required: true },
};

/** Aceita um valor fora da lista (Enter confirma o texto digitado). */
export const AllowCustomValue: Story = {
  args: {
    id: "combo-box-custom",
    titleText: "Mapa",
    placeholder: "Escolha ou digite o nome",
    items: [
      { id: "aerowalk", label: "Aerowalk" },
      { id: "bloodrun", label: "Blood Run" },
      { id: "sarnath", label: "Ruins of Sarnath" },
    ],
    allowCustomValue: true,
  },
};

/** Completa o texto com o primeiro item que começa igual (Tab aceita). */
export const Typeahead: Story = {
  args: { id: "combo-box-typeahead", typeahead: true },
};

/** Erro que veio da API (já traduzido). */
export const Invalid: Story = {
  args: {
    id: "combo-box-invalid",
    initialSelectedItem: champions[5],
    invalid: true,
    invalidText: "O Doom Slayer foi banido neste torneio.",
  },
};

export const Warn: Story = {
  args: {
    id: "combo-box-warn",
    initialSelectedItem: champions[4],
    warn: true,
    warnText: "Você já usou este campeão na série.",
  },
};

export const Disabled: Story = {
  args: {
    id: "combo-box-disabled",
    initialSelectedItem: champions[9],
    disabled: true,
  },
};

export const ReadOnly: Story = {
  args: {
    id: "combo-box-readonly",
    initialSelectedItem: champions[9],
    readOnly: true,
  },
};

/** Sobre um `Layer`, o campo troca o fundo para continuar visível. */
export const InLayer: Story = {
  args: { id: "combo-box-layer" },
  render: (args) => (
    <Layer>
      <ComboBox {...args} />
    </Layer>
  ),
};

/** Os estados lado a lado. */
export const States: Story = {
  render: (args) => (
    <Stack gap={6}>
      <ComboBox {...args} id="combo-box-states-default" />
      <ComboBox
        {...args}
        id="combo-box-states-helper"
        helperText="O campeão que você mais usa no duelo."
      />
      <ComboBox {...args} id="combo-box-states-required" required />
      <ComboBox
        {...args}
        id="combo-box-states-invalid"
        initialSelectedItem={champions[5]}
        invalid
        invalidText="O Doom Slayer foi banido neste torneio."
      />
      <ComboBox
        {...args}
        id="combo-box-states-warn"
        initialSelectedItem={champions[4]}
        warn
        warnText="Você já usou este campeão na série."
      />
      <ComboBox
        {...args}
        id="combo-box-states-disabled"
        initialSelectedItem={champions[9]}
        disabled
      />
      <ComboBox
        {...args}
        id="combo-box-states-readonly"
        initialSelectedItem={champions[9]}
        readOnly
      />
    </Stack>
  ),
};

/** Abaixo de `md` (celular), `sm` e `md` ficam com 48px de altura. */
export const Sizes: Story = {
  render: (args) => (
    <Stack gap={6}>
      {(["sm", "md", "lg"] as const).map((size) => (
        <ComboBox
          key={size}
          {...args}
          id={`combo-box-size-${size}`}
          titleText={size}
          size={size}
        />
      ))}
    </Stack>
  ),
};
