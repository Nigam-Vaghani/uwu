import { useEffect } from "react";
import { listen } from "@tauri-apps/api/event";
import { usePetStore } from "../../store/petStore";
import { QUOTE_EVENT, type QuotePayload } from "./quote.types";

export function useQuoteListener() {
  const enqueueSpeech = usePetStore((state) => state.enqueueSpeech);

  useEffect(() => {
    const unlisten = listen<QuotePayload>(QUOTE_EVENT, (event) => {
      enqueueSpeech({
        text: event.payload.text,
        tone: "warm",
        source: "quote",
        priority: 1,
      });
    });

    return () => {
      void unlisten.then((release) => release());
    };
  }, [enqueueSpeech]);
}
