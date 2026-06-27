// i18n setup (react-i18next). English is the default; pt-BR is selectable.
// The chosen language is detected from localStorage (key `jobrabbit_lang`) and
// persisted there by the language switcher in the Config page.
import i18n from "i18next";
import { initReactI18next } from "react-i18next";
import LanguageDetector from "i18next-browser-languagedetector";
import en from "./locales/en.json";
import ptBR from "./locales/pt-BR.json";

export const LANGUAGES = [
  { code: "en", label: "English" },
  { code: "pt-BR", label: "Português (Brasil)" },
] as const;

i18n
  .use(LanguageDetector)
  .use(initReactI18next)
  .init({
    resources: {
      en: { translation: en },
      "pt-BR": { translation: ptBR },
    },
    fallbackLng: "en",
    supportedLngs: ["en", "pt-BR"],
    interpolation: { escapeValue: false },
    detection: {
      order: ["localStorage", "navigator"],
      lookupLocalStorage: "jobrabbit_lang",
      caches: ["localStorage"],
    },
  });

export default i18n;
