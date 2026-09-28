import type { Kind } from "./types";

/** Offer a starting point for new work, never for edits or explanatory questions. */
export function creationKind(text: string): Kind | null {
  const value = text
    .normalize("NFD")
    .replace(/[\u0300-\u036f]/g, "")
    .toLowerCase();
  if (
    /\b(?:design_id|template_id)\s*:|\b(?:sin (?:usar )?plantillas?|desde cero|without (?:a )?template|from scratch)\b/.test(
      value,
    )
  )
    return null;
  if (
    /(?:^|[.!?¿]\s*)(?:como|how to|how do|explica\w*|explain|que es|what is)\b|\b(?:no (?:gener\w*|quiero|necesito)|do not create|don't create)\b/.test(
      value,
    )
  )
    return null;
  // Mentions of creation inside capability questions are not creation requests.
  if (
    /\b(?:funcionalidades|capacidades|habilidades|skills|capabilities|features|abilities)\b/.test(
      value,
    ) &&
    /\b(?:que|cuales|cuantas|cuantos|tienes|puedes|what|which|how many|can you)\b/.test(
      value,
    )
  )
    return null;
  if (
    /\b(?:solo|solamente|unicamente|only)\s+(?:puedes|sabes|creas|generas|can you|do you)\b|\b(?:eres capaz|sabes crear|sabes generar|are you able)\b/.test(
      value,
    )
  )
    return null;
  if (
    /\b(?:quiero|necesito|want|need)\s+(?:saber|conocer|entender|aprender|to know|to understand|to learn)\b/.test(
      value,
    )
  )
    return null;
  if (
    /^(?:[¿?\s]*)(?:puedes|can you)\s+(?:crear|generar|disenar|create|generate|design)\s+(?:presentaciones|diapositivas|documentos|pdfs?|interfaces|imagenes|videos|graficos|slides|presentations|documents|websites|images|charts)[?¿.\s]*$/.test(
      value,
    )
  )
    return null;
  const action =
    /\b(?:genera\w*|crea\w*|disena\w*|hazme|haz|prepara\w*|quiero|necesito|create|generate|design|build|make|prepare|i want|i need)\b/.exec(
      value,
    );
  if (!action) return null;
  const subject = value.slice(
    action.index + action[0].length,
    action.index + action[0].length + 180,
  );
  const categories: [Kind, RegExp][] = [
    [
      "presentation",
      /\b(?:diapositivas?|presentacion(?:es)?|pptx?|powerpoint|slides?|slide ?deck|presentations?)\b/,
    ],
    [
      "web",
      /\b(?:interfaces?|interfaz|paginas?(?: web)?|sitios? web|websites?|web ?pages?|landing(?: page)?|dashboard|ui)\b/,
    ],
    [
      "document",
      /\b(?:pdf|documentos?|documents?|informes?|reports?|word|docx)\b/,
    ],
    ["spreadsheet", /\b(?:excel|xlsx|hojas? de calculo|spreadsheets?)\b/],
    ["diagram", /\b(?:diagramas?|diagrams?|flowcharts?|flujogramas?)\b/],
    ["chart", /\b(?:graficas?|graficos?|charts?|graphs?)\b/],
    ["image", /\b(?:imagenes?|images?|carteles?|cartel|posters?|afiches?)\b/],
    ["video", /\b(?:videos?|animaciones?|animations?)\b/],
  ];
  const match = categories
    .map(([kind, pattern]) => ({ kind, index: subject.search(pattern) }))
    .filter((match) => match.index >= 0)
    .sort((a, b) => a.index - b.index)[0];
  if (!match) return null;
  if (
    /\b(?:corrig\w*|modifi\w*|edit\w*|mejor\w*|actuali\w*|cambi\w*|revis\w*|resum\w*|summariz\w*|update|fix|modify|improve|change)\b/.test(
      subject.slice(0, match.index),
    )
  )
    return null;
  return match.kind;
}
