// The interface text in the language the person chose (NFR-I18N-1). Each language file has the shape of
// strings.en.ts; screens import from here, and `chooseLanguage` switches every string at once before they render.

import * as en from "./strings.en";
import * as id from "./strings.id";

export type Language = "en" | "id";

const all: Record<Language, typeof en> = { en, id };

/** The languages the app has text for, each named in itself. */
export const LANGUAGES: { id: Language; name: string }[] = [
  { id: "en", name: "English" },
  { id: "id", name: "Bahasa Indonesia" },
];

export const KEY_ADDRESS = en.KEY_ADDRESS;
// ES module exports are live bindings: reassigning these changes them for every importer.
export let steps = en.steps;
export let keyOutcomes = en.keyOutcomes;
export let samples = en.samples;
export let t = en.t;
export let errorMessage = en.errorMessage;
export let language: Language = "en";

export function chooseLanguage(chosen: string) {
  language = chosen === "id" ? "id" : "en";
  const text = all[language];
  steps = text.steps;
  keyOutcomes = text.keyOutcomes;
  samples = text.samples;
  t = text.t;
  errorMessage = text.errorMessage;
  document.documentElement.lang = language;
}
