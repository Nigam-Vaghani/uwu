import { useCallback, useEffect, useRef } from "react";

export type SpeechQueueItem = {
  id: string;
  text: string;
  priority: number;
};

type ProcessItemFn = (item: SpeechQueueItem) => Promise<void>;

function insertByPriority(queue: SpeechQueueItem[], item: SpeechQueueItem): SpeechQueueItem[] {
  const next = [...queue, item];
  next.sort((left, right) => right.priority - left.priority);
  return next;
}

export function useSpeechQueue(processItem: ProcessItemFn) {
  const queueRef = useRef<SpeechQueueItem[]>([]);
  const processingRef = useRef(false);
  const processItemRef = useRef(processItem);
  const idRef = useRef(0);

  useEffect(() => {
    processItemRef.current = processItem;
  }, [processItem]);

  const pump = useCallback(async () => {
    if (processingRef.current) {
      return;
    }

    processingRef.current = true;

    try {
      while (queueRef.current.length > 0) {
        const [next, ...rest] = queueRef.current;
        queueRef.current = rest;
        await processItemRef.current(next);
      }
    } finally {
      processingRef.current = false;
    }
  }, []);

  const enqueue = useCallback(
    (text: string, priority = 0) => {
      const trimmed = text.trim();
      if (!trimmed) {
        return;
      }

      idRef.current += 1;
      const item: SpeechQueueItem = {
        id: `speech-${idRef.current}`,
        text: trimmed,
        priority,
      };

      queueRef.current = insertByPriority(queueRef.current, item);
      void pump();
    },
    [pump],
  );

  const clear = useCallback(async () => {
    queueRef.current = [];
  }, []);

  const size = useCallback(() => queueRef.current.length, []);

  return { enqueue, clear, size };
}
