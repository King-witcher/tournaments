import type { Meta, StoryObj } from "@storybook/nextjs-vite";
import { useState } from "react";
import { fn } from "storybook/test";

import { FormModal } from "./form-modal";
import { TextInput } from "./text-input";

// O Modal guarda o estado dos campos: quem usa o monta só enquanto aberto,
// de um componente que tem o estado (aqui, a própria história).
function NameField() {
  const [name, setName] = useState("");
  return (
    <TextInput
      labelText="Nome do time"
      required
      value={name}
      onChange={(event) => setName(event.target.value)}
    />
  );
}

const meta = {
  title: "Ação/FormModal",
  component: FormModal,
  // Sai da árvore (no body) e é fixo: tela cheia na página da história, e
  // um quadro com altura própria na documentação.
  parameters: {
    layout: "fullscreen",
    docs: { story: { inline: false, height: "500px" } },
  },
  args: {
    title: "Inscrever time",
    submitLabel: "Inscrever",
    onSubmit: fn(),
    onClose: fn(),
    children: <NameField />,
  },
  argTypes: {
    onSubmit: { control: false },
    onClose: { control: false },
    children: { control: false },
    returnFocusTo: { control: false },
    title: { control: "text" },
    submitLabel: { control: "text" },
    error: { control: "text" },
  },
} satisfies Meta<typeof FormModal>;

export default meta;
type Story = StoryObj<typeof meta>;

/** O foco começa no primeiro campo e Enter envia. */
export const Default: Story = {};

/** O primário fica travado enquanto falta preencher algo. */
export const SubmitDisabled: Story = { args: { submitDisabled: true } };

/** Ignora o clique e mostra o indicador no primário. */
export const Pending: Story = { args: { pending: true } };

/** O erro geral que a API devolveu, já traduzido, acima dos botões. */
export const WithError: Story = {
  args: { error: "Já existe um time com este nome." },
};

/** Mais largo e com "Fechar": o que já foi salvo fica ao sair. */
export const WiderWithClose: Story = {
  args: { size: "md", cancelLabel: "Fechar" },
};

/** Primário `danger` para o que tira algo. */
export const Destructive: Story = {
  args: {
    destructive: true,
    title: "Remover senha",
    submitLabel: "Remover senha",
    children: <TextInput labelText="Senha atual" type="password" required />,
  },
};
