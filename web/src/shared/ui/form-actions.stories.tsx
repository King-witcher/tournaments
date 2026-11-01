import type { Meta, StoryObj } from "@storybook/nextjs-vite";
import { fn } from "storybook/test";

import { Button } from "./button";
import { FormActions } from "./form-actions";

const meta = {
  title: "Formulário/FormActions",
  component: FormActions,
  args: { onCancel: fn() },
  argTypes: {
    submitLabel: { control: "text" },
    cancelLabel: { control: "text" },
    cancelHref: { control: false },
    onCancel: { control: false },
    submitRef: { control: false },
    extra: { control: false },
    alternative: { control: false },
  },
} satisfies Meta<typeof FormActions>;

export default meta;
type Story = StoryObj<typeof meta>;

/** O primário à esquerda (o padrão é "Salvar"), "Cancelar" secundário ao lado. */
export const Default: Story = {};

export const SubmitLabel: Story = { args: { submitLabel: "Criar torneio" } };

/** "Cancelar" como link, quando leva a outra página. */
export const CancelLink: Story = {
  args: { onCancel: undefined, cancelHref: "/", submitLabel: "Criar torneio" },
};

/** Sem `cancelHref` nem `onCancel`, não há "Cancelar". */
export const WithoutCancel: Story = { args: { onCancel: undefined } };

/** Ignora o clique e mostra o indicador, sem `disabled` (o foco fica). */
export const Pending: Story = { args: { pending: true } };

/** Enquanto faltar campo obrigatório ou houver campo inválido. */
export const Disabled: Story = { args: { disabled: true } };

export const Destructive: Story = {
  args: { destructive: true, submitLabel: "Remover senha" },
};

/** Uma ação a mais na ponta oposta. */
export const Extra: Story = {
  args: {
    extra: <Button kind="tertiary">Sortear</Button>,
  },
};

/** Duas formas de enviar: "Cancelar" vira `ghost` para os três pesos serem diferentes. */
export const Alternative: Story = {
  args: {
    submitLabel: "Criar",
    alternative: <Button kind="secondary">Criar como rascunho</Button>,
  },
};
