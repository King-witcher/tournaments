import type { ReactNode } from "react";

import { type Status, StatusTag } from "./status-tag";

const STATUS: Record<"neutral" | "accent" | "warning" | "success", Status> = {
  neutral: "neutral",
  accent: "registration",
  warning: "check_in",
  success: "confirmed",
};

/**
 * @deprecated Use `StatusTag` com o significado (`status`), não a cor. Este
 * só traduz as variantes antigas.
 */
export function Badge({
  children,
  variant = "neutral",
}: {
  children: ReactNode;
  variant?: keyof typeof STATUS;
}) {
  return (
    <StatusTag status={STATUS[variant]} size="sm">
      {children}
    </StatusTag>
  );
}
