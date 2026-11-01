import { Map as MapIcon } from "@carbon/icons-react";
import { Stack } from "@carbon/react";
import type { Meta, StoryObj } from "@storybook/nextjs-vite";
import { useState } from "react";
import { fn } from "storybook/test";

import { DismissibleTag } from "./dismissible-tag";

const types = [
  "gray",
  "cool-gray",
  "warm-gray",
  "red",
  "magenta",
  "purple",
  "blue",
  "cyan",
  "teal",
  "green",
  "high-contrast",
  "outline",
] as const;

const meta = {
  title: "Estado e conteúdo/DismissibleTag",
  component: DismissibleTag,
  args: { text: "Awoken", type: "gray", onClose: fn() },
  argTypes: {
    type: { control: "select", options: types },
    size: { control: "inline-radio", options: ["sm", "md", "lg"] },
    text: { control: "text" },
    renderIcon: { control: false },
    onClose: { control: false },
  },
} satisfies Meta<typeof DismissibleTag>;

export default meta;
type Story = StoryObj<typeof meta>;

/** Passe o mouse ou foque o X: a dica é "Remover". */
export const Default: Story = {};

export const WithIcon: Story = { args: { renderIcon: MapIcon } };

export const Disabled: Story = { args: { disabled: true } };

/** Texto cortado: a dica do X vira "Remover a etiqueta" e o texto inteiro fica no `title`. */
export const LongText: Story = {
  args: {
    text: "Church of Azathoth, só na fase de grupos da copa de inverno",
  },
};

export const Types: Story = {
  render: (args) => (
    <Stack orientation="horizontal" gap={3}>
      {types.map((type) => (
        <DismissibleTag key={type} {...args} type={type} text={type} />
      ))}
    </Stack>
  ),
};

/** Abaixo de `md` (celular), todos ficam com 48px de altura. */
export const Sizes: Story = {
  render: (args) => (
    <Stack orientation="horizontal" gap={3}>
      {(["sm", "md", "lg"] as const).map((size) => (
        <DismissibleTag key={size} {...args} size={size} text={size} />
      ))}
    </Stack>
  ),
};

function PoolDemo() {
  const [pool, setPool] = useState([
    "Awoken",
    "Blood Covenant",
    "Corrupted Keep",
    "Molten Falls",
    "Ruins of Sarnath",
  ]);
  return (
    <Stack orientation="horizontal" gap={3}>
      {pool.map((map) => (
        <DismissibleTag
          key={map}
          text={map}
          type="blue"
          onClose={() => setPool((current) => current.filter((m) => m !== map))}
        />
      ))}
    </Stack>
  );
}

/** A pool de mapas: o X tira o mapa da lista. */
export const RemoveFromList: Story = { render: () => <PoolDemo /> };
