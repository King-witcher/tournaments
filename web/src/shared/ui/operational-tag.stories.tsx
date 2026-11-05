import { Map as MapIcon } from "@carbon/icons-react";
import type { Meta, StoryObj } from "@storybook/nextjs-vite";
import { fn } from "storybook/test";

import { ListItem, UnorderedList } from "./carbon/list";
import { Stack } from "./carbon/stack";
import { OperationalTag } from "./operational-tag";
import { Popover, PopoverContent } from "./popover";

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
] as const;

const meta = {
  title: "Estado e conteúdo/OperationalTag",
  component: OperationalTag,
  args: { text: "+3 mapas", type: "gray", onClick: fn() },
  argTypes: {
    type: { control: "select", options: types },
    size: { control: "inline-radio", options: ["sm", "md", "lg"] },
    text: { control: "text" },
    renderIcon: { control: false },
    onClick: { control: false },
  },
} satisfies Meta<typeof OperationalTag>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};

export const WithIcon: Story = { args: { renderIcon: MapIcon } };

export const Disabled: Story = { args: { disabled: true } };

export const Types: Story = {
  render: (args) => (
    <Stack orientation="horizontal" gap={3}>
      {types.map((type) => (
        <OperationalTag key={type} {...args} type={type} text={type} />
      ))}
    </Stack>
  ),
};

/** Abaixo de `md` (celular), todos ficam com 48px de altura. */
export const Sizes: Story = {
  render: (args) => (
    <Stack orientation="horizontal" gap={3}>
      {(["sm", "md", "lg"] as const).map((size) => (
        <OperationalTag key={size} {...args} size={size} text={size} />
      ))}
    </Stack>
  ),
};

/** O uso típico: abre a lista que a etiqueta resume. */
export const WithPopover: Story = {
  parameters: {
    layout: "centered",
    docs: { story: { inline: false, height: "240px" } },
  },
  render: (args) => (
    <Popover trigger={<OperationalTag {...args} />} defaultOpen>
      <PopoverContent>
        <UnorderedList>
          <ListItem>Awoken</ListItem>
          <ListItem>Blood Covenant</ListItem>
          <ListItem>Molten Falls</ListItem>
        </UnorderedList>
      </PopoverContent>
    </Popover>
  ),
};
