import { useEffect, useState, type FormEvent } from "react";
import { invokeCommand } from "../../hooks/useTauriStore";
import type { CreateReminderInput, Reminder, UpdateReminderInput } from "./reminders.types";
import { CATEGORY_LABELS, REMINDER_CATEGORIES } from "./reminders.types";

type ReminderFormProps = {
  reminder?: Reminder | null;
  onSaved: () => void;
  onCancel: () => void;
};

const emptyForm: CreateReminderInput = {
  title: "",
  message: "",
  cronExpr: "@every:60",
  category: "custom",
};

export function ReminderForm({ reminder, onSaved, onCancel }: ReminderFormProps) {
  const [title, setTitle] = useState(reminder?.title ?? emptyForm.title);
  const [message, setMessage] = useState(reminder?.message ?? emptyForm.message);
  const [cronExpr, setCronExpr] = useState(reminder?.cronExpr ?? emptyForm.cronExpr);
  const [category, setCategory] = useState(reminder?.category ?? emptyForm.category);
  const [enabled, setEnabled] = useState(reminder?.enabled ?? true);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    setTitle(reminder?.title ?? emptyForm.title);
    setMessage(reminder?.message ?? emptyForm.message);
    setCronExpr(reminder?.cronExpr ?? emptyForm.cronExpr);
    setCategory(reminder?.category ?? emptyForm.category);
    setEnabled(reminder?.enabled ?? true);
  }, [reminder]);

  const handleSubmit = async (event: FormEvent) => {
    event.preventDefault();
    setSaving(true);
    setError(null);

    try {
      if (reminder) {
        const input: UpdateReminderInput = {
          id: reminder.id,
          title,
          message,
          cronExpr,
          category,
          enabled,
        };
        await invokeCommand("update_reminder", { input });
      } else {
        const input: CreateReminderInput = { title, message, cronExpr, category };
        await invokeCommand("create_reminder", { input });
      }
      onSaved();
    } catch (submitError) {
      setError(submitError instanceof Error ? submitError.message : String(submitError));
    } finally {
      setSaving(false);
    }
  };

  return (
    <form className="reminder-form" onSubmit={handleSubmit}>
      <h3>{reminder ? "Edit Reminder" : "New Reminder"}</h3>

      <label className="settings-field">
        <span>Title</span>
        <input
          type="text"
          value={title}
          required
          onChange={(event) => setTitle(event.currentTarget.value)}
        />
      </label>

      <label className="settings-field">
        <span>Message</span>
        <textarea
          value={message}
          required
          rows={3}
          onChange={(event) => setMessage(event.currentTarget.value)}
        />
      </label>

      <label className="settings-field">
        <span>Schedule</span>
        <input
          type="text"
          value={cronExpr}
          required
          placeholder="@every:30 or cron expression"
          onChange={(event) => setCronExpr(event.currentTarget.value)}
        />
      </label>

      <p className="settings-hint">
        Use <code>@every:30</code> for every 30 minutes, or a standard cron expression.
      </p>

      <label className="settings-field">
        <span>Category</span>
        <select
          value={category}
          onChange={(event) =>
            setCategory(event.currentTarget.value as CreateReminderInput["category"])
          }
        >
          {REMINDER_CATEGORIES.map((option) => (
            <option key={option} value={option}>
              {CATEGORY_LABELS[option]}
            </option>
          ))}
        </select>
      </label>

      {reminder ? (
        <label className="settings-field settings-toggle-field">
          <span>Enabled</span>
          <button
            type="button"
            className={`settings-toggle${enabled ? " settings-toggle--on" : ""}`}
            aria-pressed={enabled}
            onClick={() => setEnabled((value) => !value)}
          />
        </label>
      ) : null}

      {error ? <p className="settings-error">{error}</p> : null}

      <div className="reminder-form-actions">
        <button className="settings-secondary" type="button" onClick={onCancel} disabled={saving}>
          Cancel
        </button>
        <button className="settings-primary" type="submit" disabled={saving}>
          {saving ? "Saving..." : reminder ? "Update" : "Create"}
        </button>
      </div>
    </form>
  );
}
