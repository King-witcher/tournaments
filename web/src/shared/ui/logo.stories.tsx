import type { Meta, StoryObj } from "@storybook/nextjs-vite";

import { AppHeader } from "./app-header";
import { Logo } from "./logo";

const meta = {
  title: "Cabeçalho/Logo",
  component: Logo,
} satisfies Meta<typeof Logo>;

export default meta;
type Story = StoryObj<typeof meta>;

/** Na cor do texto: muda com o tema. */
export const Default: Story = {};

/** No cabeçalho, como link para o início. */
export const InHeader: Story = {
  parameters: {
    layout: "fullscreen",
    docs: { story: { inline: false, height: "120px" } },
  },
  render: () => (
    <AppHeader name="Telefrag" skipLabel="Pular para o conteúdo">
      {null}
    </AppHeader>
  ),
};

/** Abaixo de `md` fica só o ícone, para a barra de ações caber. */
export const Phone: Story = {
  ...InHeader,
  globals: { viewport: { value: "phone", isRotated: false } },
};
