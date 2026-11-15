import { button, next, submitKey, VALID_KEY, workspaceFile, writeWorkspaceFile } from "../helpers";

const MARATHON = "Training for the half marathon in April.";

async function say(text: string) {
  await $("aria/Message Pec").setValue(text);
  await browser.keys("Enter");
  await $(`//div[contains(@class, "bubble user") and contains(., "${text}")]`).waitForDisplayed();
}

describe("dreaming: Pec dreams what keeps coming up, and the dream can be taken back", () => {
  before(async () => {
    await submitKey(VALID_KEY);
    await next(3);
    await button("Meet Pec").click();
    await expect($("p*=Hello, Sam! (")).toBeDisplayed();
  });

  it("promotes a recalled note on dream now and tells about it in the diary", async () => {
    writeWorkspaceFile(
      "memory/2026-11-02.md",
      `# 2026-11-02\n\n- 09:00 ${MARATHON} <!-- from:companion src:1-0 -->\n`,
    );
    // Two different questions bring the note back: recall evidence for the dream.
    await say("half marathon training");
    await say("April marathon");
    await $("aria/Dreams").click();
    await button("Dream now").click();
    await expect($("p*=I dreamt of a long road")).toBeDisplayed();
    await expect($("li*=1 kept")).toBeDisplayed();
    expect(workspaceFile("MEMORY.md")).toContain(`${MARATHON} <!-- id:`);
    expect(workspaceFile("DREAMS.md")).toContain("Kept:");
  });

  it("undoes the last dream", async () => {
    await button("Undo last dream").click();
    await expect($("[role=status]")).toHaveText(expect.stringContaining("The last dream was undone"));
    expect(workspaceFile("MEMORY.md")).not.toContain(MARATHON);
  });
});
