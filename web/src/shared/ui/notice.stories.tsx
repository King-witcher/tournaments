import { Stack } from "@carbon/react";
import type { Meta, StoryObj } from "@storybook/nextjs-vite";
import { fn } from "storybook/test";

import { Button } from "./button";
import { Link } from "./link";
import { Notice } from "./notice";

const kinds = ["info", "success", "warning", "error"] as const;

const meta = {
  title: "Estado e conteúdo/Notice",
  component: Notice,
  args: { kind: "info", children: "O check-in abre 30 minutos antes." },
  argTypes: {
    kind: { control: "select", options: kinds },
    title: { control: "text" },
    action: { control: false },
    children: { control: "text" },
  },
} satisfies Meta<typeof Notice>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Info: Story = {};

export const Success: Story = {
  args: { kind: "success", children: "Inscrição feita." },
};

export const Warning: Story = {
  args: {
    kind: "warning",
    children: "O prazo para contestar termina em 10 minutos.",
  },
};

/** Lido na hora pelo leitor de tela (`role="alert"`). */
export const Failure: Story = {
  args: { kind: "error", children: "Não foi possível enviar a partida." },
};

export const WithTitle: Story = {
  args: {
    kind: "warning",
    title: "Série contestada",
    children: "A organização vai decidir o resultado.",
  },
};

/** O Carbon recusa link ou botão dentro do aviso: a ação vai logo abaixo. */
export const WithLinkAction: Story = {
  args: {
    kind: "info",
    children: "Você ainda não fez o check-in.",
    action: <Link href="/arena-sul/copa">Ir para o torneio</Link>,
  },
};

export const WithButtonAction: Story = {
  args: {
    kind: "error",
    title: "Falha ao carregar",
    children: "Não conseguimos buscar os inscritos.",
    action: (
      <Button kind="ghost" onClick={fn()}>
        Tentar de novo
      </Button>
    ),
  },
};

/** Filho que não é texto (negrito, quebra) vai como conteúdo do aviso. */
export const RichContent: Story = {
  args: {
    kind: "info",
    title: "Campo travado",
    children: (
      <>
        O rating só muda <strong>depois do início</strong>.
      </>
    ),
  },
};

export const OnlyTitle: Story = {
  args: { kind: "success", title: "Partida enviada", children: undefined },
};

export const Kinds: Story = {
  render: (args) => (
    <Stack gap={5}>
      {kinds.map((kind) => (
        <Notice key={kind} {...args} kind={kind} title={kind}>
          O check-in abre 30 minutos antes.
        </Notice>
      ))}
    </Stack>
  ),
};

export const Phone: Story = {
  args: {
    kind: "warning",
    title: "Série contestada",
    children: "A organização vai decidir o resultado em até 24 horas.",
    action: <Link href="/arena-sul/copa">Ver a série</Link>,
  },
  globals: { viewport: { value: "phone", isRotated: false } },
};
