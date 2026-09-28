interface RecognitionResult {
  isFinal: boolean;
  [index: number]: { transcript: string };
}
interface RecognitionEvent {
  resultIndex: number;
  results: ArrayLike<RecognitionResult>;
}
interface Recognition {
  lang: string;
  continuous: boolean;
  interimResults: boolean;
  onresult: ((event: RecognitionEvent) => void) | null;
  onerror: ((event: { error: string }) => void) | null;
  onend: (() => void) | null;
  start(): void;
  abort(): void;
}
type RecognitionConstructor = new () => Recognition;
declare global {
  interface Window {
    SpeechRecognition?: RecognitionConstructor;
    webkitSpeechRecognition?: RecognitionConstructor;
  }
}
export const recognitionAvailable = () =>
  Boolean(window.SpeechRecognition || window.webkitSpeechRecognition);
export function recognizer(
  language: string,
  onFinal: (text: string) => void,
  onInterim: (text: string) => void,
  onError: (error: string) => void,
  options: {
    restart?: boolean;
    onStopped?: () => void;
    onTranscript?: (text: string) => void;
  } = {},
) {
  const Constructor =
    window.SpeechRecognition || window.webkitSpeechRecognition;
  if (!Constructor)
    throw new Error(
      "Speech recognition is unavailable in this browser / El navegador no permite reconocimiento de voz",
    );
  const recognition = new Constructor();
  recognition.lang = language;
  recognition.continuous = options.restart !== false;
  recognition.interimResults = true;
  let wanted = false,
    paused = false,
    running = false,
    disposed = false,
    retry: ReturnType<typeof setTimeout> | undefined;
  const delivered = new Set<number>();
  recognition.onresult = (e) => {
    if (disposed || !wanted || paused) return;
    if (options.onTranscript) {
      // The browser sends a snapshot of the utterance, not text to append.
      const parts = Array.from(e.results, (result) =>
        result[0].transcript.trim(),
      ).filter(Boolean);
      options.onTranscript(parts.join(" "));
      return;
    }
    let interim = "";
    for (let i = e.resultIndex; i < e.results.length; i++) {
      const result = e.results[i];
      if (result.isFinal) {
        if (!delivered.has(i)) {
          delivered.add(i);
          onFinal(result[0].transcript);
        }
      } else interim += result[0].transcript;
    }
    onInterim(interim);
  };
  const start = () => {
    if (!wanted || paused || running || disposed) return;
    try {
      delivered.clear();
      recognition.start();
      running = true;
    } catch (error) {
      onError(String(error));
    }
  };
  recognition.onend = () => {
    running = false;
    if (wanted && !paused && !disposed) {
      if (options.restart !== false) retry = setTimeout(start, 400);
      else {
        wanted = false;
        options.onStopped?.();
      }
    }
  };
  recognition.onerror = (e) => {
    if (e.error === "aborted" || e.error === "no-speech") return;
    wanted = false;
    onError(e.error);
  };
  return {
    start() {
      wanted = true;
      start();
    },
    pause() {
      paused = true;
      clearTimeout(retry);
      recognition.abort();
    },
    resume() {
      paused = false;
      start();
    },
    stop() {
      wanted = false;
      clearTimeout(retry);
      recognition.abort();
    },
    dispose() {
      disposed = true;
      wanted = false;
      clearTimeout(retry);
      recognition.onend = null;
      recognition.onresult = null;
      recognition.onerror = null;
      recognition.abort();
    },
  };
}
/** Browser echo cancellation + energy detection allows interrupting paused STT during TTS. */
export async function microphoneInterrupt(
  onSpeech: () => void,
  isSpeaking: () => boolean,
) {
  const stream = await navigator.mediaDevices.getUserMedia({
    audio: {
      echoCancellation: true,
      noiseSuppression: true,
      autoGainControl: true,
    },
  });
  let context: AudioContext;
  try {
    context = new AudioContext();
    await context.resume();
  } catch (error) {
    stream.getTracks().forEach((t) => t.stop());
    throw error;
  }
  const source = context.createMediaStreamSource(stream),
    analyser = context.createAnalyser();
  analyser.fftSize = 1024;
  source.connect(analyser);
  const samples = new Float32Array(analyser.fftSize);
  let frames = 0;
  const timer = setInterval(() => {
    analyser.getFloatTimeDomainData(samples);
    const rms = Math.sqrt(
      samples.reduce((sum, x) => sum + x * x, 0) / samples.length,
    );
    frames = isSpeaking() && rms > 0.06 ? frames + 1 : 0;
    if (frames >= 5) {
      frames = 0;
      onSpeech();
    }
  }, 40);
  return () => {
    clearInterval(timer);
    source.disconnect();
    stream.getTracks().forEach((t) => t.stop());
    void context.close();
  };
}
