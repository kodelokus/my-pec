import { VALID_KEY, workspaceFile } from "../helpers";

describe("onboarding US1: from a key to a companion", () => {
  it("completes setup on the defaults with the keyboard only", async () => {
    // Keyboard only: focus the key field by tabbing, type, submit with Enter.
    const field = $("aria/Claude API key");
    while (!(await field.isFocused())) await browser.keys("Tab");
    await browser.keys([...VALID_KEY, "Enter"]);
    await expect($("p*=Key accepted")).toBeDisplayed();

    for (const label of ["Next", "Next", "Next", "Meet Pec"]) {
      const target = $(`button=${label}`);
      while (!(await target.isFocused())) await browser.keys("Tab");
      await browser.keys("Enter");
    }

    await expect($("p=Pec is here")).toBeDisplayed();
    expect(workspaceFile("IDENTITY.md")).toContain("- Name: Pec");
    for (const file of ["SOUL.md", "USER.md", "MEMORY.md", "avatar/manifest.json"]) {
      expect(workspaceFile(file).length).toBeGreaterThan(0);
    }
  });
});
