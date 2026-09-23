import { api, type TurnReply } from "./api";
import { microphoneInterrupt, recognizer } from "./speech";
export interface CallState {
  active: boolean;
  starting: boolean;
  phase: string;
  transcript: string;
  reply: string;
  error: string;
  muted: boolean;
}
export class LiveCall {
  state: CallState = {
    active: false,
    starting: false,
    phase: "idle",
    transcript: "",
    reply: "",
    error: "",
    muted: false,
  };
  private id?: string;
  private epoch = 0;
  private language = "es";
  private speech?: ReturnType<typeof recognizer>;
  private releaseMicrophone?: () => void;
  private timer?: ReturnType<typeof setInterval>;
  private utteranceTimer?: ReturnType<typeof setTimeout>;
  private events?: EventSource;
  private speaking = false;
  private speechEpoch = 0;
  private dirty = true;
  private lastUser = 0;
  private pending = "";
  private queue: string[] = [];
  private request?: AbortController;
  private observation = false;
  constructor(private changed: (state: CallState) => void) {}
  dismissError() {
    this.update({ error: "" });
  }
  private update(patch: Partial<CallState>) {
    this.state = { ...this.state, ...patch };
    this.changed({ ...this.state });
  }
  async start(language: string) {
    if (this.state.active || this.state.starting) return;
    const epoch = ++this.epoch;
    this.language = language;
    this.update({ starting: true, error: "", phase: "connecting" });
    try {
      const cleanup = await microphoneInterrupt(
        () => this.interrupt(),
        () => this.speaking && !this.state.muted,
      );
      if (epoch !== this.epoch) {
        cleanup();
        return;
      }
      this.releaseMicrophone = cleanup;
      const result = await api<{ id: string }>("/live/sessions", {});
      if (epoch !== this.epoch) {
        void api(`/live/sessions/${result.id}`, undefined, "DELETE");
        return;
      }
      this.id = result.id;
      this.speech = recognizer(
        language === "es" ? "es-PE" : "en-US",
        (text) => this.collect(text),
        (text) => {
          this.lastUser = Date.now();
          this.update({ transcript: text });
        },
        (error) => {
          this.update({ error, phase: "microphone-error", muted: true });
          this.speech?.stop();
        },
      );
      this.update({
        active: true,
        starting: false,
        phase: "listening",
        muted: false,
        transcript: "",
        reply: "",
      });
      this.speech.start();
      this.events = new EventSource("/api/events");
      this.events.addEventListener("activity", () => {
        this.dirty = true;
      });
      this.timer = setInterval(() => {
        if (
          this.dirty &&
          !this.request &&
          !this.queue.length &&
          !this.speaking &&
          Date.now() - this.lastUser > 3000
        )
          void this.turn();
      }, 20000);
      this.speak(
        language === "es"
          ? "Estoy aquí. Podemos hablar mientras el agente sigue trabajando."
          : "I’m here. We can talk while the agent keeps working.",
      );
    } catch (error) {
      if (epoch === this.epoch) {
        this.close();
        this.update({ error: String(error), phase: "error" });
      }
    }
  }
  private collect(text: string) {
    if (!this.state.active || this.state.muted || this.speaking) return;
    this.lastUser = Date.now();
    this.pending += " " + text;
    this.update({ transcript: this.pending.trim() });
    clearTimeout(this.utteranceTimer);
    this.utteranceTimer = setTimeout(() => {
      const text = this.pending.trim();
      this.pending = "";
      this.send(text);
    }, 650);
  }
  send(text: string) {
    text = text.trim();
    if (!text || !this.state.active) return;
    this.lastUser = Date.now();
    this.interrupt();
    if (this.request && this.observation) this.request.abort();
    this.queue.push(text);
    this.update({ transcript: text });
    void this.drain();
  }
  private async drain() {
    if (this.request || !this.queue.length || !this.state.active) return;
    await this.turn(this.queue.shift());
  }
  private async turn(text?: string) {
    if (!this.id || this.request) return;
    const epoch = this.epoch;
    const controller = new AbortController();
    this.request = controller;
    this.observation = text === undefined;
    this.dirty = false;
    if (text) this.update({ phase: "thinking", error: "" });
    try {
      const result = await api<TurnReply>(
        `/live/sessions/${this.id}/turn`,
        {
          request_id: crypto.randomUUID(),
          text: text ?? null,
          language: this.language,
        },
        "POST",
        controller.signal,
      );
      if (epoch !== this.epoch) return;
      window.dispatchEvent(new Event("fenix:messages"));
      if (result.reply) {
        this.update({ reply: result.reply });
        if (!this.queue.length) this.speak(result.reply);
      }
    } catch (error) {
      if (epoch === this.epoch && !controller.signal.aborted)
        this.update({ error: String(error), phase: "error" });
    } finally {
      if (epoch === this.epoch) {
        this.request = undefined;
        if (!this.speaking && !this.state.error)
          this.update({ phase: "listening" });
        void this.drain();
      }
    }
  }
  private speak(text: string) {
    if (!this.state.active) return;
    this.speech?.pause();
    this.speaking = true;
    this.update({ phase: "speaking" });
    const utterance = new SpeechSynthesisUtterance(text.slice(0, 700));
    utterance.lang = this.language === "es" ? "es-PE" : "en-US";
    const voices = speechSynthesis.getVoices();
    const voice =
      voices.find(
        (v) => v.lang.toLowerCase() === utterance.lang.toLowerCase(),
      ) || voices.find((v) => v.lang.startsWith(this.language));
    if (voice) utterance.voice = voice;
    const epoch = this.epoch;
    const speechEpoch = ++this.speechEpoch;
    const finished = () => {
      if (epoch !== this.epoch || speechEpoch !== this.speechEpoch) return;
      this.speaking = false;
      this.update({ phase: "listening" });
      if (!this.state.muted) this.speech?.resume();
    };
    utterance.onend = finished;
    utterance.onerror = finished;
    speechSynthesis.speak(utterance);
  }
  interrupt() {
    if (!this.speaking) return;
    ++this.speechEpoch;
    this.speaking = false;
    speechSynthesis.cancel();
    if (!this.state.muted) this.speech?.resume();
    this.update({ phase: "listening" });
  }
  toggleMute() {
    const muted = !this.state.muted;
    this.update({ muted });
    if (muted) {
      this.speech?.stop();
      this.pending = "";
      clearTimeout(this.utteranceTimer);
    } else {
      this.speech?.start();
    }
  }
  close() {
    ++this.epoch;
    const id = this.id;
    this.id = undefined;
    this.request?.abort();
    this.request = undefined;
    this.queue = [];
    this.pending = "";
    clearInterval(this.timer);
    clearTimeout(this.utteranceTimer);
    this.events?.close();
    this.speech?.dispose();
    this.speech = undefined;
    this.releaseMicrophone?.();
    this.releaseMicrophone = undefined;
    this.speaking = false;
    if ("speechSynthesis" in window) speechSynthesis.cancel();
    this.update({
      active: false,
      starting: false,
      phase: "idle",
      muted: false,
      transcript: "",
    });
    if (id)
      void fetch(`/api/live/sessions/${id}`, {
        method: "DELETE",
        headers: { "Content-Type": "application/json" },
        keepalive: true,
      }).catch(() => {});
  }
}
