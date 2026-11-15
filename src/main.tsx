import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { App } from "./App";
import { language, onLanguageChanged } from "./api/commands";
import { NoteWindow } from "./extensions/NoteWindow";
import { FloatingView } from "./floating/FloatingView";
import { chooseLanguage } from "./strings";
import "./styles.css";

// avatar R8: the floating window loads the same page, told before it loads that it floats and the
// companion's name, and shows only the character.
const { floatingName, noteFile } = window as { floatingName?: string; noteFile?: string };
const floating = floatingName !== undefined;
if (floating) document.documentElement.classList.add("floating");
// extensions R7: a note window is told its note before it loads.
if (noteFile !== undefined) document.documentElement.classList.add("note");

// NFR-I18N-1: the text is in the chosen language before anything is drawn; English if it can't be read.
void onLanguageChanged(() => window.location.reload());
void language()
  .catch(() => "en")
  .then((chosen) => {
    chooseLanguage(chosen);
    createRoot(document.getElementById("root") as HTMLElement).render(
      <StrictMode>
        {noteFile !== undefined ? (
          <NoteWindow file={noteFile} />
        ) : floating ? (
          <FloatingView name={floatingName} />
        ) : (
          <App />
        )}
      </StrictMode>,
    );
  });
