import type { Meta, StoryObj } from "@storybook/nextjs-vite";
import { type ComponentProps, useState } from "react";

import { MarkdownField } from "./markdown-field";

type Args = ComponentProps<typeof MarkdownField>;

/** O campo é controlado: a história guarda o texto. */
function Controlled(args: Args) {
  const [value, setValue] = useState(args.value);
  return (
    <MarkdownField
      {...args}
      value={value}
      onChange={(event) => setValue(event.target.value)}
    />
  );
}

const meta = {
  title: "Formulário/MarkdownField",
  component: MarkdownField,
  args: {
    labelText: "Regras da organização em português",
    rows: 8,
    maxLength: 20000,
    value:
      "# Conduta\nRespeite os adversários.\n\n## Queda de conexão\n- Avise no **#suporte**.\n- Volte em até 5 minutos.\n\nDetalhes no [regulamento](https://telefrag.com.br).",
  },
  argTypes: { value: { control: "text" }, onChange: { control: false } },
  render: (args) => <Controlled {...args} />,
} satisfies Meta<typeof MarkdownField>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};

/** "Visualizar" sem texto. */
export const Empty: Story = { args: { value: "" } };

/** Encerrado: só leitura, e a pré-via continua. */
export const ReadOnly: Story = { args: { readOnly: true } };

/** Erro da API: o campo volta para "Escrever", onde o erro aparece. */
export const Invalid: Story = {
  args: { invalid: true, invalidText: "O texto passou do limite." },
};

/** Com o contador do `TextArea`. */
export const WithCounter: Story = {
  args: { enableCounter: true, maxCount: 20000 },
};

export const Phone: Story = {
  globals: { viewport: { value: "phone", isRotated: false } },
};

const wait = () => new Promise((resolve) => setTimeout(resolve, 1500));

/**
 * `onImage` e `imagePrefix`: "Inserir imagem" fica pendente durante o envio
 * (simulado) e põe `![nome do arquivo](url)` no cursor, com a descrição
 * selecionada; a pré-via desenha a imagem.
 */
export const WithImages: Story = {
  args: {
    imagePrefix: "https://picsum.photos/",
    onImage: async () => {
      await wait();
      return { ok: true, url: "https://picsum.photos/seed/telefrag/960/480" };
    },
  },
};

/** O envio falhou: o erro sai embaixo do botão. */
export const ImageError: Story = {
  args: {
    imagePrefix: "https://picsum.photos/",
    onImage: async () => {
      await wait();
      return {
        ok: false,
        error: "O envio da imagem falhou. Confira a conexão e tente de novo.",
      };
    },
  },
};
