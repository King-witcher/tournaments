import type { Meta, StoryObj } from "@storybook/nextjs-vite";
import { useState } from "react";

import { Button } from "./button";
import { Stack } from "./carbon/stack";
import { Loading } from "./loading";

// Com a camada por cima (o padrão do Carbon), cobre a página: nas histórias
// ela fica desligada, menos nas que a mostram, cada uma num quadro próprio.
const overlayFrame = {
  layout: "fullscreen",
  docs: { story: { inline: false, height: "500px" } },
};

const meta = {
  title: "Estado e conteúdo/Loading",
  component: Loading,
  args: { withOverlay: false },
  argTypes: {
    description: { control: "text" },
  },
} satisfies Meta<typeof Loading>;

export default meta;
type Story = StoryObj<typeof meta>;

/** O leitor de tela lê "Carregando…". */
export const Default: Story = {};

export const Small: Story = { args: { small: true } };

/** `active={false}`: para o giro, na saída. */
export const Inactive: Story = { args: { active: false } };

/** O que está carregando, para o leitor de tela. */
export const WithDescription: Story = {
  args: { description: "Gerando a chave" },
};

/** Cobre a tela e impede o uso até terminar. */
export const WithOverlay: Story = {
  args: { withOverlay: true },
  parameters: overlayFrame,
};

function OverlayDemo() {
  const [active, setActive] = useState(false);
  const start = () => {
    setActive(true);
    setTimeout(() => setActive(false), 2000);
  };
  return (
    <>
      <Button onClick={start}>Gerar a chave</Button>
      {active && <Loading description="Gerando a chave" />}
    </>
  );
}

/** Aparece ao clicar e sai depois de 2 segundos. */
export const OverlayOnAction: Story = {
  render: () => <OverlayDemo />,
  parameters: overlayFrame,
};

export const Sizes: Story = {
  render: (args) => (
    <Stack orientation="horizontal" gap={7}>
      <Loading {...args} />
      <Loading {...args} small />
    </Stack>
  ),
};
