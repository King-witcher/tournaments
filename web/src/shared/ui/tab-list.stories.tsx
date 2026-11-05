import type { Meta, StoryObj } from "@storybook/nextjs-vite";

import { Tab, TabPanel, TabPanels, Tabs } from "./carbon/tabs";
import { TabList } from "./tab-list";

const labels = ["Visão geral", "Inscritos", "Chave", "Regras"];

const meta = {
  title: "Página/TabList",
  component: TabList,
  args: {
    "aria-label": "Seções do torneio",
    children: labels.map((label) => <Tab key={label}>{label}</Tab>),
  },
  argTypes: {
    activation: { control: "inline-radio", options: ["automatic", "manual"] },
    size: { control: "inline-radio", options: ["sm", "md", "lg"] },
    children: { control: false },
  },
  render: (args) => (
    <Tabs>
      <TabList {...args} />
    </Tabs>
  ),
} satisfies Meta<typeof TabList>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};

/** Com as abas do Carbon, `TabPanels` troca o conteúdo; em `PageTabs` cada aba é uma rota. */
export const WithPanels: Story = {
  render: (args) => (
    <Tabs>
      <TabList {...args} />
      <TabPanels>
        {labels.map((label) => (
          <TabPanel key={label}>{label}</TabPanel>
        ))}
      </TabPanels>
    </Tabs>
  ),
};

/** Setas movem o foco; Enter abre a aba (o `PageTabs` usa este modo). */
export const ManualActivation: Story = { args: { activation: "manual" } };

export const Contained: Story = { args: { contained: true } };

/** Só vale em `contained`, com menos de 9 abas. */
export const ContainedFullWidth: Story = {
  args: { contained: true, fullWidth: true },
};

export const Small: Story = { args: { size: "sm" } };

export const DisabledTab: Story = {
  args: {
    children: labels.map((label) => (
      <Tab key={label} disabled={label === "Chave"}>
        {label}
      </Tab>
    )),
  },
};

/** Abaixo de `md` não há setas de rolagem: a barra rola com o dedo. */
export const ManyTabsPhone: Story = {
  globals: { viewport: { value: "phone", isRotated: false } },
  args: {
    children: [...labels, "Mapas", "Classificação", "Organização"].map(
      (label) => <Tab key={label}>{label}</Tab>,
    ),
  },
};
