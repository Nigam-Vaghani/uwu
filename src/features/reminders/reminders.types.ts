export type ReminderCategory = "hydration" | "break" | "coding" | "custom";

export type Reminder = {
  id: number;
  title: string;
  message: string;
  cronExpr: string;
  enabled: boolean;
  category: ReminderCategory;
  lastFired: number | null;
};

export type CreateReminderInput = {
  title: string;
  message: string;
  cronExpr: string;
  category: ReminderCategory;
};

export type UpdateReminderInput = CreateReminderInput & {
  id: number;
  enabled: boolean;
};

export const REMINDER_CATEGORIES: ReminderCategory[] = [
  "hydration",
  "break",
  "coding",
  "custom",
];

export const CATEGORY_LABELS: Record<ReminderCategory, string> = {
  hydration: "Hydration",
  break: "Break",
  coding: "Coding",
  custom: "Custom",
};
