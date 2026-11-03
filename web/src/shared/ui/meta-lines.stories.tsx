import type { Meta, StoryObj } from "@storybook/nextjs-vite";

import { LinkTile } from "./link-tile";
import { MetaLines } from "./meta-lines";
import { TileGrid } from "./tile-grid";

const meta = {
  title: "Listas/MetaLines",
  component: MetaLines,
  args: {
    lines: ["Arena Sul · terça, 21h", "Próxima série: 2.1 contra Rafael", null],
  },
  argTypes: { lines: { control: false } },
  render: (args) => (
    <TileGrid label="Exemplo">
      <LinkTile
        href="/arena-sul/copa"
        title="Copa Arena Sul"
        meta={<MetaLines {...args} />}
      />
    </TileGrid>
  ),
} satisfies Meta<typeof MetaLines>;

export default meta;
type Story = StoryObj<typeof meta>;

/** Um fato por linha; o item vazio não ocupa lugar. */
export const Default: Story = {};
