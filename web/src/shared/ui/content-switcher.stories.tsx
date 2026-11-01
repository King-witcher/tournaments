import {
  type CarbonIconType,
  Grid,
  List,
  TreeViewAlt,
} from "@carbon/icons-react";
import { Layer, Stack } from "@carbon/react";
import type { Meta, StoryObj } from "@storybook/nextjs-vite";
import { useState } from "react";
import { useArgs } from "storybook/preview-api";
import { fn } from "storybook/test";

import { ContentSwitcher } from "./content-switcher";

const views = [
  { value: "bracket", text: "Chave" },
  { value: "rounds", text: "Por rodada" },
  { value: "groups", text: "Por grupo" },
];

const iconViews = [
  { value: "bracket", text: "Chave", icon: TreeViewAlt },
  { value: "list", text: "Lista", icon: List },
  { value: "grid", text: "Grade", icon: Grid },
];

const meta = {
  title: "Formulário/ContentSwitcher",
  component: ContentSwitcher,
  args: {
    "aria-label": "Visão das séries",
    options: views,
    value: "bracket",
    onChange: fn(),
  },
  argTypes: {
    size: { control: "inline-radio", options: ["sm", "md", "lg"] },
    selectionMode: {
      control: "inline-radio",
      options: ["automatic", "manual"],
    },
    options: { control: "object" },
    value: { control: "text" },
    onChange: { control: false },
  },
  // Controlado: a escolha volta para os controles da história.
  render: function Render(args) {
    const [, updateArgs] = useArgs();
    return (
      <ContentSwitcher
        {...args}
        onChange={(value) => {
          args.onChange(value);
          updateArgs({ value });
        }}
      />
    );
  },
} satisfies Meta<typeof ContentSwitcher>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};

/** Só ícones, com o nome de cada um na dica. */
export const IconOnly: Story = {
  args: { "aria-label": "Visão", options: iconViews },
};

/** Fundo mais discreto para o selecionado. */
export const LowContrast: Story = {
  args: { lowContrast: true },
};

export const LowContrastIconOnly: Story = {
  args: { "aria-label": "Visão", options: iconViews, lowContrast: true },
};

/** As setas só movem o foco; Enter ou espaço escolhem. */
export const ManualSelection: Story = {
  args: { selectionMode: "manual" },
};

export const OptionDisabled: Story = {
  args: {
    options: [...views.slice(0, 2), { ...views[2], disabled: true }],
  },
};

function Stateful(props: {
  label: string;
  options: { value: string; text: string; icon?: CarbonIconType }[];
  initial: string;
  size?: "sm" | "md" | "lg";
  lowContrast?: boolean;
}) {
  const { label, initial, ...rest } = props;
  const [value, setValue] = useState(initial);
  return (
    <ContentSwitcher
      {...rest}
      aria-label={label}
      value={value}
      onChange={setValue}
    />
  );
}

/** Sobre um `Layer`, o seletor troca o fundo para continuar visível. */
export const InLayer: Story = {
  render: () => (
    <Layer>
      <Stack gap={5}>
        <Stateful label="Visão das séries" options={views} initial="bracket" />
        <Stateful label="Visão" options={iconViews} initial="bracket" />
      </Stack>
    </Layer>
  ),
};

/** Os tamanhos e os tipos lado a lado. */
export const Variants: Story = {
  render: () => (
    <Stack gap={6}>
      {(["sm", "md", "lg"] as const).map((size) => (
        <Stateful
          key={size}
          label={`Visão das séries (${size})`}
          options={views}
          initial="bracket"
          size={size}
        />
      ))}
      <Stateful label="Visão" options={iconViews} initial="bracket" />
      <Stateful
        label="Visão das séries"
        options={views}
        initial="rounds"
        lowContrast
      />
    </Stack>
  ),
};
