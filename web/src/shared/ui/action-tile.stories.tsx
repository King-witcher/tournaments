import type { Meta, StoryObj } from "@storybook/nextjs-vite";

import { ActionTile } from "./action-tile";
import { Avatar } from "./avatar";
import { Button } from "./button";
import { ButtonLink } from "./button-link";
import { MetaLines } from "./meta-lines";
import { TileGrid } from "./tile-grid";

const meta = {
  title: "Listas/ActionTile",
  component: ActionTile,
  args: {
    title: "Copa Arena Sul",
    meta: "Arena Sul · Check-in até hoje, 21h",
    action: (
      <Button kind="tertiary" size="sm">
        Fazer check-in
      </Button>
    ),
  },
  argTypes: {
    title: { control: "text" },
    meta: { control: "text" },
    avatar: { control: false },
    action: { control: false },
  },
  render: (args) => (
    <TileGrid stacked label="Exemplo">
      <ActionTile {...args} />
    </TileGrid>
  ),
} satisfies Meta<typeof ActionTile>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};

/** Com o logo, em duas linhas de contexto e um link com cara de botão. */
export const WithLogo: Story = {
  args: {
    avatar: <Avatar shape="square" size="md" name="Arena Sul" />,
    meta: (
      <MetaLines
        lines={[
          "Arena Sul · 2.23, partida 1 (Crucible)",
          "Rafael contestou há 2 horas",
        ]}
      />
    ),
    action: (
      <ButtonLink
        href="/arena-sul/copa/matches/wb-2-23"
        kind="tertiary"
        size="sm"
      >
        Resolver
      </ButtonLink>
    ),
  },
};

/** Várias, empilhadas, como no início. */
export const Stacked: Story = {
  render: (args) => (
    <TileGrid stacked label="Exemplo">
      <ActionTile {...args} />
      <ActionTile
        title="Liga de inverno"
        meta="Quake RS · Próxima série: R4.2 contra Camila"
        action={
          <ButtonLink
            href="/quake-rs/liga/matches/rr-4-2"
            kind="tertiary"
            size="sm"
          >
            Enviar partida
          </ButtonLink>
        }
      />
    </TileGrid>
  ),
};

/** No celular, o botão desce. */
export const Phone: Story = {
  globals: { viewport: { value: "phone" } },
};
