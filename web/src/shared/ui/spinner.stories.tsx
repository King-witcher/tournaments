import type { Meta, StoryObj } from "@storybook/nextjs-vite";

import { Spinner } from "./spinner";

const meta = {
  title: "Estado e conteúdo/Spinner",
  component: Spinner,
  args: { label: "Carregando" },
  argTypes: { label: { control: "text" }, className: { control: false } },
} satisfies Meta<typeof Spinner>;

export default meta;
type Story = StoryObj<typeof meta>;

/** Só o indicador; o `label` é lido pelo leitor de tela. Dentro de um botão pendente, veja `Button` > `Pending`. */
export const Default: Story = {};
