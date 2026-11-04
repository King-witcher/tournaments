"use client";

import { useTranslations } from "next-intl";
import { useEffect, useId, useRef, useState } from "react";

import { Button } from "@/shared/ui/button";
import { UserText } from "@/shared/ui/user-text";
import styles from "./tenant-description.module.scss";

const LINES = 2;

/**
 * A descrição da comunidade no cabeçalho: até duas linhas e, só se o texto
 * passar disso (medido, também ao mudar a largura), "Ver mais", que mostra o
 * resto ali mesmo, e "Ver menos".
 */
export function TenantDescription({ children }: { children: string }) {
  const t = useTranslations("Tenant.description");
  const id = useId();
  const text = useRef<HTMLDivElement>(null);
  const [expanded, setExpanded] = useState(false);
  const [overflows, setOverflows] = useState(false);

  // biome-ignore lint/correctness/useExhaustiveDependencies: o texto novo precisa ser medido de novo.
  useEffect(() => {
    const element = text.current;
    if (!element) return;
    const measure = () => {
      const line = Number.parseFloat(getComputedStyle(element).lineHeight);
      // Expandido, a altura toda do texto; recolhido, a que o corte esconde.
      setOverflows(element.scrollHeight > line * LINES + 1);
    };
    measure();
    const observer = new ResizeObserver(measure);
    observer.observe(element);
    return () => observer.disconnect();
  }, [children]);

  return (
    <div className={styles.description}>
      <UserText id={id} ref={text} clamp={!expanded}>
        {children}
      </UserText>
      {overflows && (
        <Button
          kind="ghost"
          size="sm"
          className={styles.toggle}
          aria-expanded={expanded}
          aria-controls={id}
          onClick={() => setExpanded((open) => !open)}
        >
          {expanded ? t("less") : t("more")}
        </Button>
      )}
    </div>
  );
}
