import type { Meta, StoryObj } from "@storybook/nextjs-vite";

import { LinkTile } from "./link-tile";
import { Page } from "./page";
import { PageColumns } from "./page-columns";
import { Section } from "./section";
import { TileGrid } from "./tile-grid";

const tiles = ["Copa Arena Sul", "Duelo de sexta", "Liga de inverno", "2v2"];

const meta = {
  title: "Página/PageColumns",
  component: PageColumns,
  args: {
    main: (
      <Section title="Seus torneios">
        <TileGrid>
          {tiles.map((title) => (
            <LinkTile
              key={title}
              href="/arena-sul"
              title={title}
              meta="Arena Sul"
            />
          ))}
        </TileGrid>
      </Section>
    ),
    aside: (
      <Section title="Suas comunidades">
        <TileGrid>
          <LinkTile href="/arena-sul" title="Arena Sul" meta="Dono" />
          <LinkTile href="/quake-rs" title="Quake RS" meta="Jogador" />
        </TileGrid>
      </Section>
    ),
  },
  argTypes: { main: { control: false }, aside: { control: false } },
  render: (args) => (
    <Page width="wide">
      <PageColumns {...args} />
    </Page>
  ),
} satisfies Meta<typeof PageColumns>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};

/** Abaixo do `lg`, uma coluna embaixo da outra. */
export const Phone: Story = {
  globals: { viewport: { value: "phone" } },
};
