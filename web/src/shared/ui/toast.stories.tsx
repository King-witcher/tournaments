import type { Meta, StoryObj } from "@storybook/nextjs-vite";
import { useState } from "react";
import { fn, userEvent, within } from "storybook/test";

import { Button } from "./button";
import { Toast } from "./toast";

const meta = {
  title: "Estado e conteúdo/Toast",
  component: Toast,
  // Fixo no canto de cima: um quadro com altura própria na documentação.
  parameters: {
    layout: "fullscreen",
    docs: { story: { inline: false, height: "500px" } },
  },
  // `timeout: 0` mantém o aviso na tela para ver; o padrão é 5000 ms.
  args: { open: true, title: "Link copiado", timeout: 0, onClose: fn() },
  argTypes: {
    kind: { control: "inline-radio", options: ["success", "info"] },
    onClose: { control: false },
  },
} satisfies Meta<typeof Toast>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Success: Story = {};

export const Info: Story = {
  args: { kind: "info", title: "Check-in aberto" },
};

function CopyDemo() {
  const [open, setOpen] = useState(false);
  return (
    <>
      <Button kind="tertiary" onClick={() => setOpen(true)}>
        Copiar link
      </Button>
      <Toast
        open={open}
        onClose={() => setOpen(false)}
        title="Link copiado"
        timeout={5000}
      />
    </>
  );
}

/** Aparece ao clicar e some sozinho depois de 5 segundos. */
export const Timeout: Story = {
  render: () => <CopyDemo />,
  play: async ({ canvasElement }) => {
    await userEvent.click(within(canvasElement).getByRole("button"));
  },
};
