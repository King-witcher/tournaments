import type { Meta, StoryObj } from "@storybook/nextjs-vite";
import { useCallback, useState } from "react";
import { fn, userEvent, within } from "storybook/test";

import { Button } from "./button";
import { Link } from "./link";
import { type ToastItem, ToastStack } from "./toast-stack";

const items: ToastItem[] = [
  {
    id: "1",
    title: "Série pronta",
    subtitle: "Sua série começa em 5 minutos.",
  },
  {
    id: "2",
    title: "Check-in aberto",
    subtitle: "Faça o check-in para entrar no torneio.",
    action: <Link href="/login">Ir para o check-in</Link>,
  },
  { id: "3", title: "Resultado enviado" },
];

const meta = {
  title: "Estado e conteúdo/ToastStack",
  component: ToastStack,
  // Fixa no canto de baixo: um quadro com altura própria na documentação.
  parameters: {
    layout: "fullscreen",
    docs: { story: { inline: false, height: "500px" } },
  },
  // O tempo longo mantém os avisos na tela para ver.
  args: {
    items,
    label: "Avisos",
    timeout: 600000,
    onClose: fn(),
  },
  argTypes: {
    items: { control: "object" },
    onClose: { control: false },
  },
} satisfies Meta<typeof ToastStack>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};

export const Single: Story = { args: { items: items.slice(0, 1) } };

/** Com `action`, o link fica colado embaixo do aviso. */
export const WithAction: Story = { args: { items: items.slice(1, 2) } };

/** Sem itens a região (`role="log"`) continua montada, para o leitor de tela anunciar o que entra. */
export const Empty: Story = { args: { items: [] } };

function ArrivalDemo() {
  const [shown, setShown] = useState<ToastItem[]>([]);
  const close = useCallback(
    (id: string) => setShown((all) => all.filter((item) => item.id !== id)),
    [],
  );
  const arrive = () =>
    setShown((all) => [
      ...all,
      { ...items[all.length % items.length], id: String(Date.now()) },
    ]);
  return (
    <>
      <Button kind="tertiary" onClick={arrive}>
        Chegar aviso
      </Button>
      <ToastStack items={shown} onClose={close} label="Avisos" timeout={8000} />
    </>
  );
}

/** Os avisos entram sozinhos, fecham pelo X e somem em 8 segundos (parados enquanto o ponteiro ou o foco estão na pilha). */
export const Arriving: Story = {
  render: () => <ArrivalDemo />,
  play: async ({ canvasElement }) => {
    const arrive = within(canvasElement).getByRole("button");
    await userEvent.click(arrive);
    await userEvent.click(arrive);
  },
};
