/** Keep operator provisioning failures out of customer-facing error messages. */
export function serviceError(value: unknown): string {
  const text = value instanceof Error ? value.message : String(value);
  if (
    /Configure the google OAuth application \(client_(id|secret)\) first/i.test(
      text,
    )
  ) {
    return document.documentElement.lang === "en"
      ? "Google connection is not available yet. The administrator needs to finish setting it up before you can connect your account."
      : "La conexión con Google todavía no está disponible. El administrador debe terminar de configurarla para que puedas conectar tu cuenta.";
  }
  if (
    /api[ _-]?key|openai.{0,20}(key|clave)|(key|clave).{0,20}openai|client[ _-]?(secret|id)|missing.{0,20}credentials|credentials.{0,20}(missing|required|invalid)/i.test(
      text,
    )
  ) {
    return document.documentElement.lang === "en"
      ? "This service is temporarily unavailable. Please contact support if the problem continues."
      : "Este servicio no está disponible temporalmente. Contacta con soporte si el problema continúa.";
  }
  return text;
}
