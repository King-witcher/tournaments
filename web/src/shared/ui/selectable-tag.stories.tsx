import { Trophy } from "@carbon/icons-react";
import { FormGroup, Stack } from "@carbon/react";
import type { Meta, StoryObj } from "@storybook/nextjs-vite";
import { useState } from "react";
import { fn } from "storybook/test";

import { SelectableTag } from "./selectable-tag";

const meta = {
  title: "Estado e conteúdo/SelectableTag",
  component: SelectableTag,
  args: { text: "Duelo", onChange: fn() },
  argTypes: {
    size: { control: "inline-radio", options: ["sm", "md", "lg"] },
    text: { control: "text" },
    renderIcon: { control: false },
    onChange: { control: false },
  },
} satisfies Meta<typeof SelectableTag>;

export default meta;
type Story = StoryObj<typeof meta>;

/** Clique para ligar e desligar (`aria-pressed`). */
export const Default: Story = {};

export const Selected: Story = { args: { defaultSelected: true } };

export const WithIcon: Story = { args: { renderIcon: Trophy } };

export const Disabled: Story = { args: { disabled: true } };

/** Abaixo de `md` (celular), todos ficam com 48px de altura. */
export const Sizes: Story = {
  render: (args) => (
    <Stack orientation="horizontal" gap={3}>
      {(["sm", "md", "lg"] as const).map((size) => (
        <SelectableTag key={size} {...args} size={size} text={size} />
      ))}
    </Stack>
  ),
};

const modes = ["Duelo", "FFA", "2v2", "3v3", "TDM"];

function FilterDemo() {
  const [selected, setSelected] = useState<string[]>(["Duelo"]);
  const toggle = (mode: string, on: boolean) =>
    setSelected((current) =>
      on ? [...current, mode] : current.filter((m) => m !== mode),
    );
  return (
    // O grupo dá o nome comum aos filtros.
    <FormGroup legendText="Modos">
      <Stack orientation="horizontal" gap={3}>
        {modes.map((mode) => (
          <SelectableTag
            key={mode}
            text={mode}
            selected={selected.includes(mode)}
            onChange={(on: boolean) => toggle(mode, on)}
          />
        ))}
      </Stack>
    </FormGroup>
  );
}

/** Filtros de escolha múltipla, controlados de fora. */
export const Group: Story = { render: () => <FilterDemo /> };
