import type { Meta, StoryObj } from "@storybook/nextjs-vite";

import { LinkTile } from "./link-tile";
import { StatusTag } from "./status-tag";
import { TileGrid } from "./tile-grid";

const tiles = [
  <LinkTile
    key="copa"
    href="/arena-sul/copa"
    title="Copa Arena Sul"
    meta="Arena Sul · começa hoje, 21h"
    tags={<StatusTag status="in_progress">Em andamento</StatusTag>}
  />,
  <LinkTile
    key="duelo"
    href="/arena-sul/duelo"
    title="Duelo de sexta"
    meta="Quake RS · 15/11, 20h"
    tags={<StatusTag status="registration">Inscrições abertas</StatusTag>}
  />,
  <LinkTile
    key="liga"
    href="/quake-rs/liga"
    title="Liga de inverno"
    meta="Quake RS · 20/11, 19h"
    tags={<StatusTag status="check_in">Check-in</StatusTag>}
  />,
  <LinkTile
    key="2v2"
    href="/arena-sul/2v2"
    title="Mata-mata 2v2"
    meta="Arena Sul · 01/12, 21h"
    tags={<StatusTag status="draft">Rascunho</StatusTag>}
  />,
  <LinkTile
    key="sul"
    href="/arena-sul/sul"
    title="Copa do Sul"
    meta="Arena Sul · encerrado em 03/10"
    tags={<StatusTag status="finished">Encerrado</StatusTag>}
  />,
];

const meta = {
  title: "Listas/TileGrid",
  component: TileGrid,
  args: { label: "Seus torneios", children: tiles },
  argTypes: { children: { control: false } },
} satisfies Meta<typeof TileGrid>;

export default meta;
type Story = StoryObj<typeof meta>;

/** Quantas colunas couberem na largura que a grade tem. */
export const Default: Story = {};

export const SingleItem: Story = { args: { children: tiles[0] } };

/** O que não é elemento (`false`, `null`) não vira item. */
export const IgnoresEmptyChildren: Story = {
  args: { children: [false, tiles[1], null, tiles[2]] },
};

export const Phone: Story = {
  globals: { viewport: { value: "phone", isRotated: false } },
};
