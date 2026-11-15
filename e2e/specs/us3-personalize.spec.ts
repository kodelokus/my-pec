import { button, next, submitKey, VALID_KEY, workspaceFile } from "../helpers";

describe("onboarding US3: personality, name and face", () => {
  it("records every choice", async () => {
    await submitKey(VALID_KEY);
    await next();
    await $("aria/Name").setValue("Ada");
    await next();
    await $("aria/Pip").click();
    await next();
    await $("aria/What Ada calls you").setValue("Sam");
    await $("aria/Language for our conversations").setValue("English");
    await button("Meet Ada").click();

    await expect($("h1*=, Sam")).toBeDisplayed();
    await expect($("p=Ada is here")).toBeDisplayed();
    expect(workspaceFile("USER.md")).toContain("- Call them: Sam\n- Preferred language: English");
    expect(workspaceFile("avatar/manifest.json")).toContain('"character": "pip"');
  });
});
