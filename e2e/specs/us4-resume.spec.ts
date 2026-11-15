import { existsSync, readFileSync } from "node:fs";
import { join } from "node:path";
import { APP_ENV, next, submitKey, VALID_KEY } from "../helpers";

const statePath = join(APP_ENV.MYPEC_E2E_ROOT, "state", "state.json");

describe("onboarding US4: resume", () => {
  it("reopens on the soul step with the text and the key intact", async () => {
    await submitKey(VALID_KEY);
    await next();
    await $("aria/Personality text").addValue(" Typed before quitting.");
    // Quit once the last keystroke has been saved, as it is a moment after typing.
    await browser.waitUntil(
      () => existsSync(statePath) && readFileSync(statePath, "utf8").includes("Typed before quitting."),
    );

    await browser.reloadSession();

    await expect($("aria/Personality text")).toHaveValue(expect.stringContaining("Typed before quitting."));
    await expect($(".caption")).toHaveText("Step 2 of 4 · Personality");
    await $("button=Back").click();
    await expect($("p*=Key accepted")).toBeDisplayed();
  });
});
