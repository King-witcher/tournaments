import { Add } from "@carbon/icons-react";
import type { Meta, StoryObj } from "@storybook/nextjs-vite";
import { expect, fn, userEvent, within } from "storybook/test";

import { Avatar } from "./avatar";
import { Button } from "./button";
import {
  DataTable,
  type DataTableColumn,
  type DataTableRow,
} from "./data-table";
import { OverflowActions } from "./overflow-actions";
import { StatusTag } from "./status-tag";

const NAMES = [
  "João Silva",
  "Ana Paula",
  "Lobo",
  "Mariana",
  "Rafael",
  "Camila",
  "Thiago",
  "Beatriz",
  "Gustavo",
  "Larissa",
];

const columns: DataTableColumn[] = [
  { key: "player", header: "Jogador" },
  { key: "community", header: "Comunidade", secondary: true },
  { key: "rating", header: "Rating", secondary: true },
  { key: "status", header: "Situação" },
];

const STATUSES = [
  { status: "confirmed", label: "Check-in feito" },
  { status: "check_in", label: "Check-in pendente" },
  { status: "registration", label: "Inscrito" },
  { status: "cancelled", label: "Desistiu" },
] as const;

/** `count` jogadores de exemplo, com o nome na busca. */
function players(count: number): DataTableRow[] {
  return Array.from({ length: count }, (_, i) => {
    const name = `${NAMES[i % NAMES.length]} ${Math.floor(i / NAMES.length) + 1}`;
    const state = STATUSES[i % STATUSES.length];
    return {
      id: `player-${i}`,
      search: name,
      cells: {
        player: (
          <>
            <Avatar name={name} /> {name}
          </>
        ),
        community: i % 3 === 0 ? "Arena Sul" : "Quake RS",
        rating: String(1200 + ((i * 37) % 500)),
        status: <StatusTag status={state.status}>{state.label}</StatusTag>,
      },
    };
  });
}

const meta = {
  title: "Listas/DataTable",
  component: DataTable,
  args: {
    columns,
    rows: players(60),
    label: "Jogadores inscritos",
    title: "Inscritos",
    description: "Quem entrou no torneio.",
  },
  argTypes: {
    size: { control: "inline-radio", options: ["sm", "md", "lg"] },
    columns: { control: false },
    rows: { control: false },
    title: { control: "text" },
    description: { control: "text" },
    toolbar: { control: false },
    empty: { control: false },
  },
} satisfies Meta<typeof DataTable>;

export default meta;
type Story = StoryObj<typeof meta>;

/** Com mais de 25 linhas, a paginação aparece embaixo (25, 50 ou 100 por página). */
export const Default: Story = {};

/** Sem acento e sem maiúsculas: "joao" acha "João". Sem resultado, a tabela diz o que buscou. */
export const NoResults: Story = {
  play: async ({ canvasElement }) => {
    const canvas = within(canvasElement);
    const search = canvas.getByRole("searchbox");
    await userEvent.type(search, "zzz");
    await expect(search).toHaveValue("zzz");
  },
};

/** Com `href`, a célula principal vira link e a linha toda abre a página. */
export const RowLinks: Story = {
  args: {
    title: "Torneios",
    description: undefined,
    label: "Torneios da comunidade",
    columns: [
      { key: "name", header: "Torneio" },
      { key: "start", header: "Início", secondary: true },
      { key: "status", header: "Situação" },
    ],
    rows: [
      ["Copa Arena Sul", "12/11, 21h", "in_progress", "Em andamento"],
      ["Duelo de sexta", "15/11, 20h", "registration", "Inscrições abertas"],
      ["Liga de inverno", "20/11, 19h", "check_in", "Check-in"],
      ["Mata-mata 2v2", "01/12, 21h", "draft", "Rascunho"],
      ["Copa do Sul", "03/10, 21h", "finished", "Encerrado"],
    ].map(([name, start, status, label], i) => ({
      id: `torneio-${i}`,
      search: name,
      href: `/arena-sul/${i}`,
      cells: {
        name,
        start,
        status: <StatusTag status={status as "draft"}>{label}</StatusTag>,
      },
    })),
  },
};

/** Barra com a ação primária e o ⋯ por linha (última coluna, sem título). */
export const WithActions: Story = {
  args: {
    rows: players(30).map((row, i) => ({
      ...row,
      actions: (
        <OverflowActions
          items={[
            { label: "Ver perfil", onSelect: fn() },
            {
              label: "Remover",
              destructive: true,
              onSelect: fn(),
              confirm: { question: `Remover ${NAMES[i % NAMES.length]}?` },
            },
          ]}
        />
      ),
    })),
    toolbar: (
      <Button renderIcon={Add} onClick={fn()}>
        Inscrever
      </Button>
    ),
  },
};

/** Linha baixa, para tabela com muitas linhas. */
export const Small: Story = { args: { size: "sm" } };

export const Large: Story = { args: { size: "lg" } };

/** Sem título nem descrição: só a barra e a tabela. */
export const WithoutTitle: Story = {
  args: { title: undefined, description: undefined },
};

/** Poucas linhas: sem paginação. */
export const FewRows: Story = { args: { rows: players(5) } };

/** Sem linhas: o estado e o próximo passo; a ação da barra continua nela. */
export const Empty: Story = {
  args: {
    rows: [],
    empty: {
      text: "Nenhum inscrito.",
      action: <Button onClick={fn()}>Inscrever</Button>,
    },
  },
};

/** Sem linhas e sem `empty`: só a barra, quando a ação dela basta. */
export const EmptyToolbarOnly: Story = {
  args: {
    rows: [],
    toolbar: (
      <Button renderIcon={Add} onClick={fn()}>
        Gerar link
      </Button>
    ),
  },
};

/** No celular, as colunas `secondary` (comunidade e rating) somem. */
export const Phone: Story = {
  args: { rows: players(30) },
  globals: { viewport: { value: "phone", isRotated: false } },
};
