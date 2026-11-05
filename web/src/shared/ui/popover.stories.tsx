import { Filter, Settings } from "@carbon/icons-react";
import type { Meta, StoryObj } from "@storybook/nextjs-vite";
import { useState } from "react";
import { fn, userEvent, within } from "storybook/test";

import { Button } from "./button";
import { ListItem, UnorderedList } from "./carbon/list";
import { Stack } from "./carbon/stack";
import { Checkbox } from "./checkbox";
import { IconButton } from "./icon-button";
import { OperationalTag } from "./operational-tag";
import { Popover, PopoverContent } from "./popover";

const alignments = [
  "top",
  "top-start",
  "top-end",
  "bottom",
  "bottom-start",
  "bottom-end",
  "left",
  "left-start",
  "left-end",
  "right",
  "right-start",
  "right-end",
] as const;

const maps = (
  <PopoverContent>
    <UnorderedList>
      <ListItem>Awoken</ListItem>
      <ListItem>Blood Covenant</ListItem>
      <ListItem>Molten Falls</ListItem>
    </UnorderedList>
  </PopoverContent>
);

// A caixa sai por cima da página: um quadro com altura própria na
// documentação, com espaço em volta do botão.
const frame = {
  layout: "centered",
  docs: { story: { inline: false, height: "280px" } },
};

const meta = {
  title: "Estado e conteúdo/Popover",
  component: Popover,
  parameters: frame,
  args: {
    trigger: <Button kind="ghost">+3 mapas</Button>,
    children: maps,
    defaultOpen: true,
    onOpenChange: fn(),
  },
  argTypes: {
    align: { control: "select", options: alignments },
    trigger: { control: false },
    children: { control: false },
    open: { control: false },
    onOpenChange: { control: false },
  },
} satisfies Meta<typeof Popover>;

export default meta;
type Story = StoryObj<typeof meta>;

/** Embaixo, com a seta e a sombra (os padrões do Carbon). */
export const Default: Story = {};

/** Fechado: abre no clique; fecha no clique fora, no Esc e quando o foco sai. */
export const Closed: Story = {
  args: { defaultOpen: false },
  play: async ({ canvasElement }) => {
    await userEvent.click(within(canvasElement).getByRole("button"));
  },
};

export const Top: Story = { args: { align: "top" } };

export const Right: Story = { args: { align: "right" } };

export const BottomStart: Story = { args: { align: "bottom-start" } };

export const NoCaret: Story = { args: { caret: false } };

export const NoDropShadow: Story = { args: { dropShadow: false } };

export const HighContrast: Story = { args: { highContrast: true } };

export const Border: Story = { args: { border: true, dropShadow: false } };

/** Botão só com ícone como gatilho. */
export const IconTrigger: Story = {
  args: {
    trigger: (
      <IconButton label="Filtros">
        <Filter />
      </IconButton>
    ),
  },
};

function DisplayOptions() {
  const [eliminated, setEliminated] = useState(true);
  const [scores, setScores] = useState(false);
  return (
    <PopoverContent>
      <Stack gap={3}>
        <Checkbox
          labelText="Mostrar eliminados"
          checked={eliminated}
          onChange={setEliminated}
        />
        <Checkbox
          labelText="Mostrar placar mapa a mapa"
          checked={scores}
          onChange={setScores}
        />
      </Stack>
    </PopoverContent>
  );
}

/** Formato de aba, colado ao botão (um `<button>` simples, que o Carbon estiliza). */
export const TabTip: Story = {
  args: {
    isTabTip: true,
    align: "bottom-end",
    trigger: (
      <button type="button" aria-label="Opções de exibição">
        <Settings />
      </button>
    ),
    children: <DisplayOptions />,
  },
};

/** Sem espaço em cima, a caixa troca de lado sozinha. */
export const AutoAlign: Story = {
  args: { align: "top", autoAlign: true },
  parameters: { layout: "padded" },
};

function ControlledDemo() {
  const [open, setOpen] = useState(false);
  return (
    <Popover
      open={open}
      onOpenChange={setOpen}
      trigger={<OperationalTag text="+3 mapas" type="blue" />}
    >
      {maps}
    </Popover>
  );
}

/** `open` + `onOpenChange` de fora, com uma `OperationalTag` como gatilho. */
export const Controlled: Story = {
  render: () => <ControlledDemo />,
};
