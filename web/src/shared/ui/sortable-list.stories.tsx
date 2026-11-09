import type { Meta, StoryObj } from "@storybook/nextjs-vite";
import { type ComponentProps, useState } from "react";
import { fn } from "storybook/test";

import { Button } from "./button";
import { PassiveModal } from "./passive-modal";
import { SortableList } from "./sortable-list";

type Team = { id: string; name: string };

const NAMES = [
  "Lobos do Sul",
  "Estrela Vermelha",
  "Os Imortais",
  "Quad Damage",
  "Railgun Club",
  "Nordeste Arena",
  "Fênix",
  "Sem Respawn",
];

function teams(count: number): Team[] {
  return Array.from({ length: count }, (_, index) => ({
    id: String(index + 1),
    name: NAMES[index] ?? `Time ${index + 1}`,
  }));
}

type Args = ComponentProps<typeof SortableList<Team>>;

/** Guarda a ordem, como a tela que usa a lista faria. */
function Sortable(args: Args) {
  const [items, setItems] = useState(args.items);
  return (
    <SortableList
      {...args}
      items={items}
      onChange={(next) => {
        setItems(next);
        args.onChange(next);
      }}
    />
  );
}

const meta = {
  title: "Listas/SortableList",
  component: SortableList<Team>,
  args: {
    items: teams(5),
    itemLabel: (team) => team.name,
    renderItem: (team, index) => `${index + 1}. ${team.name}`,
    onChange: fn(),
  },
  argTypes: {
    items: { control: false },
    itemLabel: { control: false },
    renderItem: { control: false },
    onChange: { control: false },
    disabled: { control: "boolean" },
  },
  render: (args) => <Sortable {...args} />,
} satisfies Meta<typeof SortableList<Team>>;

export default meta;
type Story = StoryObj<typeof meta>;

/** Lista curta: pegue pela alça, ou Tab até ela e Espaço para pegar pelo teclado. */
export const Default: Story = {};

/** Maior que a tela: perto da borda, a página rola sozinha. */
export const Long: Story = { args: { items: teams(40) } };

/** Só consulta (o torneio já começou): sem a alça. */
export const Disabled: Story = { args: { disabled: true } };

/** Só a alça arrasta: tocar no resto da linha rola a página. */
export const Phone: Story = {
  args: { items: teams(30) },
  globals: { viewport: { value: "phone" } },
};

function InModal(args: Args) {
  const [open, setOpen] = useState(true);
  return (
    <>
      <Button onClick={() => setOpen(true)}>Ordenar os empatados</Button>
      {open && (
        <PassiveModal title="Empatados" onClose={() => setOpen(false)}>
          <Sortable {...args} />
        </PassiveModal>
      )}
    </>
  );
}

/** O corpo do Modal rola sozinho, a imagem fica acima dele e Esc cancela sem fechar. */
export const InsideModal: Story = {
  args: { items: teams(24) },
  parameters: {
    layout: "fullscreen",
    docs: { story: { inline: false, height: "500px" } },
  },
  render: (args) => <InModal {...args} />,
};
