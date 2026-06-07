import { useCallback, useEffect, useState } from "react";
import { invokeCommand } from "../../hooks/useTauriStore";
import { ReminderForm } from "./ReminderForm";
import type { Reminder } from "./reminders.types";
import { CATEGORY_LABELS } from "./reminders.types";

export function ReminderList() {
  const [reminders, setReminders] = useState<Reminder[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [editing, setEditing] = useState<Reminder | null>(null);
  const [creating, setCreating] = useState(false);

  const loadReminders = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const items = await invokeCommand("get_reminders", {});
      setReminders(items);
    } catch (loadError) {
      setError(loadError instanceof Error ? loadError.message : String(loadError));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    void loadReminders();
  }, [loadReminders]);

  const handleToggle = async (id: number) => {
    await invokeCommand("toggle_reminder", { id });
    await loadReminders();
  };

  const handleDelete = async (id: number) => {
    await invokeCommand("delete_reminder", { id });
    if (editing?.id === id) {
      setEditing(null);
    }
    await loadReminders();
  };

  const handleSaved = async () => {
    setCreating(false);
    setEditing(null);
    await loadReminders();
  };

  if (creating || editing) {
    return (
      <ReminderForm
        reminder={editing}
        onSaved={handleSaved}
        onCancel={() => {
          setCreating(false);
          setEditing(null);
        }}
      />
    );
  }

  return (
    <div className="settings-section">
      <div className="settings-section-header">
        <h2>Reminders</h2>
        <button className="settings-primary" type="button" onClick={() => setCreating(true)}>
          Add Reminder
        </button>
      </div>

      <p className="settings-hint">
        Hydration and break reminders are included by default. Toggle, edit, or add your own.
      </p>

      {loading ? <p className="settings-hint">Loading reminders...</p> : null}
      {error ? <p className="settings-error">{error}</p> : null}

      <ul className="reminder-list">
        {reminders.map((reminder) => (
          <li key={reminder.id} className="reminder-item">
            <div className="reminder-item-main">
              <strong>{reminder.title}</strong>
              <span className="reminder-meta">
                {CATEGORY_LABELS[reminder.category]} · {reminder.cronExpr}
              </span>
              <p>{reminder.message}</p>
            </div>
            <div className="reminder-item-actions">
              <button
                type="button"
                className={`settings-toggle${reminder.enabled ? " settings-toggle--on" : ""}`}
                aria-label={`Toggle ${reminder.title}`}
                aria-pressed={reminder.enabled}
                onClick={() => void handleToggle(reminder.id)}
              />
              <button
                className="settings-secondary"
                type="button"
                onClick={() => setEditing(reminder)}
              >
                Edit
              </button>
              <button
                className="settings-secondary"
                type="button"
                onClick={() => void handleDelete(reminder.id)}
              >
                Delete
              </button>
            </div>
          </li>
        ))}
      </ul>
    </div>
  );
}
