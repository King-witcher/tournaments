import type { Meta, StoryObj } from "@storybook/nextjs-vite";

import { LeaveGuard } from "./leave-guard";

const meta = {
  title: "Formulário/LeaveGuard",
  component: LeaveGuard,
  args: { unsaved: true, path: "/" },
  argTypes: { text: { control: "text" } },
  render: (args) => (
    <>
      <LeaveGuard {...args} />
      <a href="/outra-pagina">Clique para sair sem salvar</a>
    </>
  ),
} satisfies Meta<typeof LeaveGuard>;

export default meta;
type Story = StoryObj<typeof meta>;

/** Com mudança não salva, o link abre a pergunta. */
export const WithChanges: Story = {};

/** O texto próprio, dizendo o que se perde. */
export const CustomText: Story = {
  args: { text: "As mudanças não salvas em Geral e Formato se perdem." },
};
