import type { Meta, StoryObj } from "@storybook/nextjs-vite";
import Image from "next/image";

import { Avatar } from "./avatar";
import { Link } from "./link";
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
    footer: { control: false },
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
        <StatusTag status="in_progress">Em andamento</StatusTag>
        <StatusTag status="neutral">Duelo</StatusTag>
      </>
    ),
  },
};

export const OnlyTitle: Story = { args: { meta: undefined } };

/** O link à parte fica colado ao tile, fora do link dele. */
export const WithFooter: Story = {
  args: {
    tags: <StatusTag status="in_progress">Em andamento</StatusTag>,
    footer: <Link href="/arena-sul/copa/series/12">Série pronta</Link>,
  },
};

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
    tags: <StatusTag status="registration">Inscrições abertas</StatusTag>,
    media: <Image src="/maps/crucible.webp" alt="" fill unoptimized />,
  },
};

export const LongTitle: Story = {
  args: {
    title:
      "Campeonato Gaúcho de Quake Champions de Duelo, Temporada de Inverno",
    meta: "Associação Gaúcha de Quake · começa em 20/11, 19h",
    tags: <StatusTag status="registration">Inscrições abertas</StatusTag>,
  },
};

/** Com e sem tags e rodapé: todos esticam até a altura da linha. */
export const InGrid: Story = {
  render: () => (
    <TileGrid label="Seus torneios">
      <LinkTile
        href="/arena-sul/copa"
        title="Copa Arena Sul"
        meta="Arena Sul · hoje, 21h"
        tags={<StatusTag status="in_progress">Em andamento</StatusTag>}
        footer={<Link href="/arena-sul/copa/series/12">Série pronta</Link>}
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
        tags={<StatusTag status="check_in">Check-in</StatusTag>}
      />
    </TileGrid>
  ),
};

export const Phone: Story = {
  ...WithFooter,
  globals: { viewport: { value: "phone", isRotated: false } },
};
