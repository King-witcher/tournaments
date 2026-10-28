"use client";

import { BrightnessContrast, Moon, Sun } from "@carbon/icons-react";
import { ContentSwitcher, IconSwitch } from "@carbon/react";
import { useTranslations } from "next-intl";
import { useId } from "react";

import { useTheme } from "@/shared/hooks/use-theme";
import { THEME_CHOICES } from "@/shared/lib/theme";
import styles from "./theme-switcher.module.scss";

// Os ícones do seletor de tema do tutorial do Carbon.
const ICONS = { light: Sun, system: BrightnessContrast, dark: Moon };

/**
 * O tema de cor, no painel da pessoa (`UserPanel`), como o seletor de tema
 * do tutorial do Carbon: `ContentSwitcher` só com ícones (claro, o do
 * navegador, escuro), cada um com a dica do nome. Aplica na hora e fica
 * guardado no navegador.
 */
export function ThemeSwitcher() {
  const t = useTranslations("ThemeSwitcher");
  const { theme, setTheme } = useTheme();
  const labelId = useId();

  return (
    <div className={styles.theme}>
      <p id={labelId} className={styles.label}>
        {t("label")}
      </p>
      <ContentSwitcher
        aria-labelledby={labelId}
        selectedIndex={THEME_CHOICES.indexOf(theme)}
        onChange={({ index }) => {
          if (index !== undefined) setTheme(THEME_CHOICES[index]);
        }}
      >
        {THEME_CHOICES.map((choice) => {
          const Icon = ICONS[choice];
          return (
            <IconSwitch
              key={choice}
              name={choice}
              text={t(`choices.${choice}`)}
              align="bottom"
            >
              <Icon />
            </IconSwitch>
          );
        })}
      </ContentSwitcher>
    </div>
  );
}
