import { Layer, Stack } from "@carbon/react";
import type { Meta, StoryObj } from "@storybook/nextjs-vite";
import { useState } from "react";
import { useArgs } from "storybook/preview-api";
import { fn } from "storybook/test";

import { SelectableTile } from "./selectable-tile";

const meta = {
  title: "Formulário/SelectableTile",
  component: SelectableTile,
  args: {
    id: "selectable-tile",
    children: "Exigir captura de tela do placar",
    selected: false,
    onChange: fn(),
  },
  argTypes: {
    selected: { control: "boolean" },
    children: { control: "text" },
    onChange: { control: false },
  },
  // Controlado: a marcação volta para os controles da história.
  render: function Render(args) {
    const [, updateArgs] = useArgs();
    return (
      <SelectableTile
        {...args}
        onChange={(selected) => {
          args.onChange(selected);
          updateArgs({ selected });
        }}
      />
    );
  },
} satisfies Meta<typeof SelectableTile>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};

export const Selected: Story = {
  args: { id: "selectable-tile-selected", selected: true },
};

export const Disabled: Story = {
  args: { id: "selectable-tile-disabled", disabled: true },
};

export const DisabledSelected: Story = {
  args: {
    id: "selectable-tile-disabled-selected",
    selected: true,
    disabled: true,
  },
};

const rules = [
  { id: "screenshot", text: "Exigir captura de tela do placar" },
  { id: "checkin", text: "Exigir check-in antes da série" },
  { id: "champions", text: "Não repetir campeão na série" },
];

function Stateful({ id, text }: { id: string; text: string }) {
  const [selected, setSelected] = useState(id === "checkin");
  return (
    <SelectableTile
      id={`selectable-tile-many-${id}`}
      selected={selected}
      onChange={setSelected}
    >
      {text}
    </SelectableTile>
  );
}

/** Vários tiles, cada um uma escolha independente. */
export const MultiSelect: Story = {
  render: () => (
    <Stack gap={3}>
      {rules.map((rule) => (
        <Stateful key={rule.id} {...rule} />
      ))}
    </Stack>
  ),
};

/** Sobre um `Layer`, o tile troca o fundo para continuar visível. */
export const InLayer: Story = {
  render: () => (
    <Layer>
      <Stack gap={3}>
        {rules.map((rule) => (
          <Stateful key={rule.id} {...rule} />
        ))}
      </Stack>
    </Layer>
  ),
};
