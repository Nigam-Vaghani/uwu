import type { PetEventPayload, PetEventType } from "../events/eventEngine";

const AI_EVENT_TYPES = new Set<PetEventType>([
  "HighCpu",
  "LowBattery",
  "CodingSessionStarted",
  "IdleDetected",
  "UserInteraction",
]);

export function isAiEnabledEvent(eventType: PetEventType): boolean {
  return AI_EVENT_TYPES.has(eventType);
}

export function toAiPayload(payload: PetEventPayload) {
  return {
    eventType: payload.eventType,
    title: payload.title ?? null,
    message: payload.message ?? null,
    category: payload.category ?? null,
    reminderId: payload.reminderId ?? null,
    value: payload.value ?? null,
    appName: payload.appName ?? null,
  };
}
