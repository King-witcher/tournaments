import { Stack } from "@carbon/react";
import type { Meta, StoryObj } from "@storybook/nextjs-vite";

import { Avatar } from "./avatar";

// Sem fotos de jogador no projeto, a imagem de um mapa serve de amostra.
const image = "/maps/crucible.webp";

const sizes = ["sm", "md", "lg"] as const;

const meta = {
  title: "Estado e conteúdo/Avatar",
  component: Avatar,
  args: { name: "João Silva", size: "md" },
  argTypes: {
    size: { control: "inline-radio", options: sizes },
    src: { control: "text" },
    name: { control: "text" },
  },
  // O avatar é decorativo: o nome sempre aparece ao lado, como na tabela.
  render: (args) => (
    <Stack orientation="horizontal" gap={3}>
      <Avatar {...args} />
      <span>{args.name ?? "Sem nome"}, jogador da comunidade Arena Sul</span>
    </Stack>
  ),
} satisfies Meta<typeof Avatar>;

export default meta;
type Story = StoryObj<typeof meta>;

/** Sem imagem, a inicial do nome. */
export const Initial: Story = {};

export const WithImage: Story = { args: { src: image } };

/** A inicial é a primeira letra ou número: "[AR] Lobo" dá "A". */
export const InitialSkipsSymbols: Story = { args: { name: "[AR] Lobo" } };

/** O emoji não é partido ao meio. */
export const InitialWithEmoji: Story = { args: { name: "🔥 Inferno" } };

/** Sem nome nem imagem, o círculo fica vazio. */
export const Empty: Story = { args: { name: undefined } };

/** `src` nulo (jogador sem foto) cai na inicial. */
export const NullImage: Story = { args: { src: null } };

export const Sizes: Story = {
  render: (args) => (
    <Stack gap={5}>
      {[undefined, image].map((src) => (
        <Stack key={src ?? "initial"} gap={3}>
          <span>{src ? "Com imagem" : "Com inicial"}: sm, md e lg</span>
          <Stack orientation="horizontal" gap={3}>
            {sizes.map((size) => (
              <Avatar key={size} {...args} src={src} size={size} />
            ))}
          </Stack>
        </Stack>
      ))}
    </Stack>
  ),
};
