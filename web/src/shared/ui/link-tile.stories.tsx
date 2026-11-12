import type { Meta, StoryObj } from "@storybook/nextjs-vite";
import Image from "next/image";

import { Avatar } from "./avatar";
import { LinkTile } from "./link-tile";
import { StatusTag } from "./status-tag";
import { TileGrid } from "./tile-grid";

const meta = {
  title: "Listas/LinkTile",
  component: LinkTile,
  args: {
    href: "/arena-sul/copa",
    title: "Copa Arena Sul",
    meta: "Arena Sul · começa hoje, 21h",
  },
  argTypes: {
    href: { control: "text" },
    title: { control: "text" },
    meta: { control: "text" },
    tags: { control: false },
    avatar: { control: false },
    media: { control: false },
  },
  // O tile vai sempre dentro do TileGrid, que dá a largura de uma coluna.
  render: (args) => (
    <TileGrid label="Exemplo">
      <LinkTile {...args} />
    </TileGrid>
  ),
} satisfies Meta<typeof LinkTile>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};

export const WithTags: Story = {
  args: {
    tags: (
      <>
        <StatusTag status="cancelled">Desistiu</StatusTag>
        <StatusTag status="neutral">Rascunho</StatusTag>
      </>
    ),
  },
};

export const OnlyTitle: Story = { args: { meta: undefined } };

/** O logo da comunidade antes do nome. */
export const WithAvatar: Story = {
  args: {
    title: "Arena Sul",
    meta: "Owner",
    avatar: <Avatar shape="square" size="md" name="Arena Sul" />,
  },
};

/** A capa no topo, em 2:1. */
export const WithMedia: Story = {
  args: {
    tags: <StatusTag status="neutral">Rascunho</StatusTag>,
    media: <Image src="/maps/crucible.webp" alt="" fill unoptimized />,
  },
};

export const LongTitle: Story = {
  args: {
    title:
      "Campeonato Gaúcho de Quake Champions de Duelo, Temporada de Inverno",
    meta: "Associação Gaúcha de Quake · começa em 20/11, 19h",
    tags: <StatusTag status="neutral">Rascunho</StatusTag>,
  },
};

/** Com e sem tags: todos esticam até a altura da linha. */
export const InGrid: Story = {
  render: () => (
    <TileGrid label="Seus torneios">
      <LinkTile
        href="/arena-sul/copa"
        title="Copa Arena Sul"
        meta="Arena Sul · hoje, 21h"
        tags={<StatusTag status="cancelled">Desistiu</StatusTag>}
      />
      <LinkTile
        href="/arena-sul/duelo"
        title="Duelo de sexta"
        meta="Arena Sul · 15/11, 20h"
      />
      <LinkTile
        href="/quake-rs/liga"
        title="Liga de inverno"
        meta="Quake RS · 20/11, 19h"
        tags={<StatusTag status="neutral">Reserva, sem time</StatusTag>}
      />
    </TileGrid>
  ),
};

export const Phone: Story = {
  ...WithTags,
  globals: { viewport: { value: "phone", isRotated: false } },
};
