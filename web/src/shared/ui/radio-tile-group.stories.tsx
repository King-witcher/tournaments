import type { Meta, StoryObj } from "@storybook/nextjs-vite";
import { useState } from "react";
import { useArgs } from "storybook/preview-api";
import { fn } from "storybook/test";

import { Layer } from "./carbon/layer";
import { Stack } from "./carbon/stack";
import { RadioTileGroup } from "./radio-tile-group";

const formats = [
  { value: "single", content: "Eliminação simples" },
  { value: "double", content: "Eliminação dupla" },
  { value: "round-robin", content: "Todos contra todos" },
  { value: "swiss", content: "Suíço" },
];

/** Com o que cada opção faz, para comparar antes de escolher. */
const described = [
  {
    value: "single",
    content: "Eliminação simples",
    description: "Quem perde uma série está fora.",
  },
  {
    value: "double",
    content: "Eliminação dupla",
    description:
      "Quem perde cai para a repescagem e, se perder de novo, está fora.",
  },
  {
    value: "round-robin",
    content: "Todos contra todos",
    description: "Cada um enfrenta todos os outros uma vez.",
  },
];

const meta = {
  title: "Formulário/RadioTileGroup",
  component: RadioTileGroup,
  args: {
    legend: "Formato do torneio",
    name: "radio-tile-group",
    options: formats,
    value: "single",
    onChange: fn(),
  },
  argTypes: {
    options: { control: "object" },
    value: { control: "text" },
    onChange: { control: false },
  },
  // Controlado: a escolha volta para os controles da história.
  render: function Render(args) {
    const [, updateArgs] = useArgs();
    return (
      <RadioTileGroup
        {...args}
        onChange={(value) => {
          args.onChange(value);
          updateArgs({ value });
        }}
      />
    );
  },
} satisfies Meta<typeof RadioTileGroup>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};

/** Nada escolhido antes da primeira escolha. */
export const NoSelection: Story = {
  args: { name: "radio-tile-group-empty", value: undefined },
};

export const Required: Story = {
  args: {
    name: "radio-tile-group-required",
    value: undefined,
    required: true,
  },
};

export const Disabled: Story = {
  args: { name: "radio-tile-group-disabled", disabled: true },
};

export const OptionDisabled: Story = {
  args: {
    name: "radio-tile-group-option-disabled",
    options: [...formats.slice(0, 3), { ...formats[3], disabled: true }],
  },
};

export const WithDescription: Story = {
  args: { name: "radio-tile-group-described", options: described },
};

/**
 * Em fileiras: até quatro lado a lado, da mesma altura, quantas a largura do
 * grupo comporta (duas numa coluna de 8, uma no celular).
 */
export const Columns: Story = {
  args: {
    name: "radio-tile-group-columns",
    options: [
      ...described,
      {
        value: "swiss",
        content: "Suíço",
        description:
          "Em cada rodada, se enfrenta quem tem a mesma pontuação, sem repetir adversário.",
      },
    ],
    columns: 4,
  },
};

/**
 * Um desenho de exemplo: na largura do tile, com a mesma altura em todos, e o
 * traço de 1px em `currentColor` que não estica com o quadro.
 */
function Lines({ d }: { d: string }) {
  return (
    <svg
      width="100%"
      height={40}
      viewBox="0 0 80 40"
      preserveAspectRatio="none"
      aria-hidden="true"
    >
      <path
        d={d}
        fill="none"
        stroke="currentColor"
        vectorEffect="non-scaling-stroke"
      />
    </svg>
  );
}

/**
 * Com um desenho entre o nome e a frase de cada tile (`media`), alinhado
 * entre os tiles da fileira; some abaixo do `md`.
 */
export const WithMedia: Story = {
  args: {
    name: "radio-tile-group-media",
    options: described.map((option, i) => ({
      ...option,
      media: (
        <Lines
          d={
            [
              "M.5 5.5h24v8H.5M.5 25.5h24v8H.5M24.5 9.5h26v20h-26M50.5 19.5h29",
              "M.5 3.5h24v8H.5M24.5 7.5h28M.5 21.5h16v8H.5M52.5 7.5v18M52.5 16.5h27",
              "M20.5 4.5h40v32h-40z",
            ][i]
          }
        />
      ),
    })),
    columns: 3,
  },
};

/** Só leitura: só a escolhida, num tile sem interação. */
export const ReadOnly: Story = {
  args: {
    name: "radio-tile-group-read-only",
    options: described,
    value: "double",
    readOnly: true,
  },
};

/**
 * Só leitura com `columns` e desenho: o tile fica na largura de coluna que
 * teria na fileira, sem esticar o desenho; numa coluna só, ele some.
 */
export const ReadOnlyColumns: Story = {
  args: {
    name: "radio-tile-group-read-only-columns",
    options: described.map((option) => ({
      ...option,
      media: (
        <Lines d="M.5 5.5h24v8H.5M.5 25.5h24v8H.5M24.5 9.5h26v20h-26M50.5 19.5h29" />
      ),
    })),
    value: "double",
    readOnly: true,
    columns: 3,
  },
};

/** No celular, uma coluna, com o texto quebrando dentro do tile. */
export const Phone: Story = {
  args: { name: "radio-tile-group-phone", options: described },
  globals: { viewport: { value: "phone" } },
};

function Stateful({ name }: { name: string }) {
  const [value, setValue] = useState<string | undefined>("double");
  return (
    <RadioTileGroup
      legend="Formato do torneio"
      name={name}
      options={formats}
      value={value}
      onChange={setValue}
    />
  );
}

/** Sobre um `Layer`, os tiles trocam o fundo para continuar visíveis. */
export const InLayer: Story = {
  render: () => (
    <Layer>
      <Stateful name="radio-tile-group-layer" />
    </Layer>
  ),
};

/** Os estados lado a lado. */
export const States: Story = {
  render: () => (
    <Stack gap={7}>
      <Stateful name="radio-tile-group-states-default" />
      <RadioTileGroup
        legend="Formato do torneio"
        name="radio-tile-group-states-disabled"
        options={formats}
        value="single"
        onChange={fn()}
        disabled
      />
    </Stack>
  ),
};
